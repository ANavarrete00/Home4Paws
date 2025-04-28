use reqwest::Client;
use serde_json::Value;
use std::error::Error;

//structer for animal information
pub struct AnimalData {
    pub name: String,
    pub breed: String,
    pub description: String,
}

//function retrives token from petfinders api. This token will be used to access api effecently.
pub async fn get_token(client_id: &str, client_secret: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let res = client
        .post("https://api.petfinder.com/v2/oauth2/token")
        .form(&params)
        .send()
        .await?;

    let json: Value = res.json().await?;
    let token = json["access_token"]
        .as_str()
        .ok_or("Token not found")?
        .to_string();

    Ok(token)
}

//function to retreve data from api. api is in json format.
pub async fn get_near_animals(location: &str, token: &str, page: u32) -> Result<Vec<AnimalData>, Box<dyn Error>> {
    let url = format!( 
        "https://api.petfinder.com/v2/animals?location={}&page={}",
        location, page
    );

    let client = Client::new();
    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        println!("Failed to fetch animals: {}", response.status());
        return Ok(vec![]); //return empty list if failed
    }

    let body: Value = response.json().await?;
    let  mut animals: Vec<AnimalData> = Vec::new();

    if let Some(animals_list) = body["animals"].as_array() {
        for animal in animals_list {
            let name = animal["name"].as_str().unwrap_or("Unnamed").to_string();
            let breed = animal["breeds"]["primary"].as_str().unwrap_or("Unknown").to_string();
            let description = animal["description"].as_str().unwrap_or("No description").to_string();
            animals.push(AnimalData { name, breed, description });
        }
    } else {
        println!("No animals found.");
    }

    Ok(animals)
}