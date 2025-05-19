use reqwest::Client;
use serde_json::Value;
use std::error::Error;

//structer for animal information
#[derive(Clone)]
pub struct AnimalData {
    pub name: String,
    pub breed: String,
    pub description: String,
    pub age: String,
    pub size: String,
    pub url: String,
    pub city: String,
    pub state: String,
    pub good_with: String,
    pub photo_url: Option<String>,
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
pub async fn get_near_animals(location: &str, token: &str, page: &u32) -> Result<Vec<AnimalData>, Box<dyn Error>> {
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

    let json_response: Value = response.json().await?;
    let  mut animals: Vec<AnimalData> = Vec::new();

    if let Some(animals_list) = json_response["animals"].as_array() {
        for animal in animals_list {
            //allocate each animal to their struct
            let name = animal["name"].as_str().unwrap_or("Unnamed").to_string();
            let breed = animal["breeds"]["primary"].as_str().unwrap_or("Unknown").to_string();
            let age = animal["age"].as_str().unwrap_or("Unknown").to_string();
            let size = animal["size"].as_str().unwrap_or("Unknown").to_string();
            let url = animal["url"].as_str().unwrap_or("Unknown").to_string();
            let description = animal["description"].as_str().unwrap_or("No description").to_string();
            let city = animal["contact"]["address"]["city"].as_str().unwrap_or("Unknown").to_string();
            let state = animal["contact"]["address"]["state"].as_str().unwrap_or("Unkown").to_string();

            let gw_children = animal["good_with_children"].as_bool().unwrap_or(false);
            let gw_dogs = animal["good_with_dogs"].as_bool().unwrap_or(false);
            let gw_cats = animal["good_with_cats"].as_bool().unwrap_or(false);
            let good_with = get_good_with(gw_children, gw_dogs, gw_cats);
            let photo_url = animal["photos"]
                .as_array().and_then(|photos| photos.first())
                .and_then(|photo| photo["medium"].as_str()).map(|s| s.to_string());
            animals.push(AnimalData { name, breed, description, age, size, url, photo_url, city, state, good_with });
        }
    } else {
        println!("No animals found.");
    }

    Ok(animals)
}

pub fn get_good_with(children: bool, dogs: bool, cats: bool) -> String{
    let mut good_with = "Good with: ".to_owned();
    if !children && !dogs && !cats {
        good_with.clear();
    }
    if children {
        good_with.push_str("children");
    }
    if dogs {
        if children {
            good_with.push_str(", ");
        }
        good_with.push_str("dogs");
    }
    if cats {
        if children && dogs {
            good_with.push_str(", and");
        }
        else if children || dogs {
            good_with.push_str("and ");
        }
        good_with.push_str("cats");
    }

    good_with
}