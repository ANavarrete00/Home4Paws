# 🐾 Home4Paws

**Home4Paws** is a desktop application built with Rust and `eframe`/`egui`, designed to help users browse adoptable pets through a clean and responsive interface. It integrates with a public pet adoption API to provide real-time data on available pets, their details, and locations.

## Features

- Search and view adoptable pets with photos and details
- Filter by pet type (dog, cat, etc.), location, or breed
- Clean and responsive native UI using `egui`
- Built-in API key management (with local environment variable support)
- First-time Rust + GUI project to explore native app development

## Screenshot
![alt text](https://github.com/ANavarrete00/Home4Paws/blob/main/src/assets/githubImg.png "Screenshot of app")

## Getting Started

### Prerequisites

- Rust (latest stable)  
  [Install Rust](https://www.rust-lang.org/tools/install)

### Clone the repo

```terminal
git clone https://github.com/ANavarrete00/home4paws.git
cd home4paws
```

### API Key

- retrive an API from https://www.petfinder.com/developers/ by creating an account.
- Create a .env file in the project root and add your API key:
```env
API_KEY = your_api_key_here
API_TOKEN = your_token_here
```

### Build and Run

```terminal
cargo run
```

## Tech Stack

- Language: Rust
- GUI Framework: egui via eframe
- HTTP Requests: reqwest
- JSON Parsing: serde and serde_json
- Environment Config: dotenv

## Inspiration

This project was created to combine my interest in software development with a meaningful cause
helping people discover and adopt pets from shelters. It also serves as a hands-on learning
experience for building native desktop applications in Rust.
