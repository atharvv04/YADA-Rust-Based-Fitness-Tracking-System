use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// Simple type for food ID
type FoodId = String;
type UserId = String;


// Trait to represent a food data source (e.g., website API, XML file, etc.)
trait FoodDataSource {
    fn fetch_food_data(&self) -> Vec<Food>;
}

// Dummy web source for proof of conmcept
struct DummyWebSource;

impl FoodDataSource for DummyWebSource {
    fn fetch_food_data(&self) -> Vec<Food> {
        // While extending to handle an additional website (assignment says so), we will download and parse the data.
        // Here we just return a vector with one sample basic food.
        vec![
            Food::new_basic(
                "dummy_apple",
                "Dummy Apple",
                vec!["apple".to_string(), "fruit".to_string()],
                90,
            )
        ]
    }
}

// trait for computing target calories which can add new methods by implementing the trait and updating the mapping in one place (or even by dynamic registration)
trait CalorieCalculator {
    fn calculate(&self, profile: &UserProfile) -> u32;
}

struct HarrisBenedictCalculator;

impl CalorieCalculator for HarrisBenedictCalculator {
    fn calculate(&self, profile: &UserProfile) -> u32 {
        let bmr = match profile.gender {
            Gender::Male => 88.362 + (13.397 * profile.weight) + (4.799 * profile.height) - (5.677 * profile.age as f64),
            _ => 447.593 + (9.247 * profile.weight) + (3.098 * profile.height) - (4.330 * profile.age as f64),
        };
        (bmr * profile.activity_level.factor()) as u32
    }
}

struct MifflinStJeorCalculator;

impl CalorieCalculator for MifflinStJeorCalculator {
    fn calculate(&self, profile: &UserProfile) -> u32 {
        let bmr = match profile.gender {
            Gender::Male => (10.0 * profile.weight) + (6.25 * profile.height) - (5.0 * profile.age as f64) + 5.0,
            _ => (10.0 * profile.weight) + (6.25 * profile.height) - (5.0 * profile.age as f64) - 161.0,
        };
        (bmr * profile.activity_level.factor()) as u32
    }
}


// Enumeration for activity levels
#[derive(Debug, Clone, Copy, PartialEq)]
enum ActivityLevel {
    Sedentary,
    LightlyActive,
    ModeratelyActive,
    VeryActive,
    ExtremelyActive,
}

impl ActivityLevel {
    fn factor(&self) -> f64 {
        match self {
            ActivityLevel::Sedentary => 1.2,
            ActivityLevel::LightlyActive => 1.375,
            ActivityLevel::ModeratelyActive => 1.55,
            ActivityLevel::VeryActive => 1.725,
            ActivityLevel::ExtremelyActive => 1.9,
        }
    }
    
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sedentary" => Some(ActivityLevel::Sedentary),
            "lightly" => Some(ActivityLevel::LightlyActive),
            "moderately" => Some(ActivityLevel::ModeratelyActive),
            "very" => Some(ActivityLevel::VeryActive),
            "extremely" => Some(ActivityLevel::ExtremelyActive),
            _ => None,
        }
    }
}

// Gender enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
enum Gender {
    Male,
    Female,
    Other,
}

impl Gender {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "male" | "m" => Some(Gender::Male),
            "female" | "f" => Some(Gender::Female),
            _ => Some(Gender::Other),
        }
    }
}

// User profile structure
#[derive(Clone, Debug)]
struct UserProfile {
    username: String,
    gender: Gender,
    height: f64,  // in cm
    age: u32,
    weight: f64,  // in kg
    activity_level: ActivityLevel,
    calculation_method: String,
}

impl UserProfile {
    fn new(username: String, gender: Gender, height: f64, age: u32, weight: f64, 
           activity_level: ActivityLevel) -> Self {
        UserProfile {
            username,
            gender,
            height,
            age,
            weight,
            activity_level,
            calculation_method: "harris-benedict".to_string(),
        }
    }
    
    fn get_target_calories(&self) -> u32 {
        match self.calculation_method.as_str() {
            "harris-benedict" => HarrisBenedictCalculator.calculate(self),
            "mifflin-st-jeor" => MifflinStJeorCalculator.calculate(self),
            _ => 2000, // Default value if method not recognized
        }
    }    
    
    fn set_calculation_method(&mut self, method: &str) {
        self.calculation_method = method.to_string();
    }
    
    fn to_string(&self) -> String {
        format!("{},{:?},{},{},{},{:?},{}", 
                self.username, self.gender, self.height, self.age, self.weight, 
                self.activity_level, self.calculation_method)
    }
    
    fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 7 {
            println!("Not enough parts in profile string, got {}: {:?}", parts.len(), parts);
            return None;
        }
        
        // Debug: Print the parts
        println!("Debug - Profile parts: {:?}", parts);
        
        let username = parts[0].to_string();
        
        // The issue might be in parsing the Gender
        let gender = match parts[1] {
            "Male" | "male" => Gender::Male,
            "Female" | "female" => Gender::Female,
            _ => Gender::Other,
        };
        
        // Similarly for ActivityLevel
        let activity_level = match parts[5] {
            "Sedentary" => ActivityLevel::Sedentary,
            "LightlyActive" => ActivityLevel::LightlyActive,
            "ModeratelyActive" => ActivityLevel::ModeratelyActive,
            "VeryActive" => ActivityLevel::VeryActive,
            "ExtremelyActive" => ActivityLevel::ExtremelyActive,
            _ => {
                println!("Invalid activity level: {}", parts[5]);
                return None;
            }
        };
        
        // Parse the numeric values with better error handling
        let height = match parts[2].parse::<f64>() {
            Ok(val) => val,
            Err(e) => {
                println!("Failed to parse height: {} - {}", parts[2], e);
                return None;
            }
        };
        
        let age = match parts[3].parse::<u32>() {
            Ok(val) => val,
            Err(e) => {
                println!("Failed to parse age: {} - {}", parts[3], e);
                return None;
            }
        };
        
        let weight = match parts[4].parse::<f64>() {
            Ok(val) => val,
            Err(e) => {
                println!("Failed to parse weight: {} - {}", parts[4], e);
                return None;
            }
        };
        
        Some(UserProfile {
            username,
            gender,
            height,
            age,
            weight,
            activity_level,
            calculation_method: parts[6].to_string(),
        })
    }
}

// Basic food structure
#[derive(Debug, Clone)]
struct Food {
    id: FoodId,
    name: String,
    keywords: Vec<String>,
    calories_per_serving: u32,
    is_composite: bool,
    components: Vec<(FoodId, u32)>, // (food_id, servings) pairs for composite foods
}

impl Food {
    fn new_basic(id: &str, name: &str, keywords: Vec<String>, calories: u32) -> Self {
        Food {
            id: id.to_string(),
            name: name.to_string(),
            keywords,
            calories_per_serving: calories,
            is_composite: false,
            components: Vec::new(),
        }
    }
    
    fn new_composite(id: &str, name: &str, keywords: Vec<String>, components: Vec<(FoodId, u32)>) -> Self {
        Food {
            id: id.to_string(),
            name: name.to_string(),
            keywords,
            calories_per_serving: 0, // Will be calculated later when database is available
            is_composite: true,
            components,
        }
    }
    
    fn calculate_calories(&mut self, database: &FoodDatabase) {
        if !self.is_composite {
            return; // Basic foods already have calories set
        }
        
        let mut total_calories = 0;
        for (food_id, servings) in &self.components {
            if let Some(component_food) = database.get_food(food_id) {
                total_calories += component_food.calories_per_serving * servings;
            }
        }
        
        self.calories_per_serving = total_calories;
    }
    
    fn matches_keywords(&self, search_keywords: &[String], match_all: bool) -> bool {
        if search_keywords.is_empty() {
            return true;
        }
        
        let food_keywords: Vec<String> = self.keywords.iter().map(|s| s.to_lowercase()).collect();
        
        if match_all {
            search_keywords.iter().all(|k| {
                let k_lower = k.to_lowercase();
                food_keywords.iter().any(|fw| fw.contains(&k_lower))
            })
        } else {
            search_keywords.iter().any(|k| {
                let k_lower = k.to_lowercase();
                food_keywords.iter().any(|fw| fw.contains(&k_lower))
            })
        }
    }
    
    
    fn to_string(&self) -> String {
        let food_type = if self.is_composite { "composite" } else { "basic" };
        let keywords_str = self.keywords.join("|");
        
        if !self.is_composite {
            format!("{},{},{},{},{}", food_type, self.id, self.name, keywords_str, self.calories_per_serving)
        } else {
            let components_str = self.components.iter()
                .map(|(id, servings)| format!("{}:{}", id, servings))
                .collect::<Vec<_>>()
                .join("|");
            
            format!("{},{},{},{},{}", food_type, self.id, self.name, keywords_str, components_str)
        }
    }
}

// Food entry for daily log
#[derive(Debug, Clone)]
struct FoodEntry {
    food_id: FoodId,
    servings: u32,
    timestamp: u64,
}

impl FoodEntry {
    fn new(food_id: &str, servings: u32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        FoodEntry {
            food_id: food_id.to_string(),
            servings,
            timestamp,
        }
    }
    
    fn to_string(&self) -> String {
        format!("{},{},{}", self.food_id, self.servings, self.timestamp)
    }
    
    fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 3 {
            return None;
        }
        
        let food_id = parts[0].to_string();
        let servings = parts[1].parse::<u32>().ok()?;
        let timestamp = parts[2].parse::<u64>().ok()?;
        
        Some(FoodEntry {
            food_id,
            servings,
            timestamp,
        })
    }
}

// Command for undo functionality
enum CommandType {
    AddFood(String, FoodEntry),     // (date, entry)
    DeleteFood(String, FoodEntry),  // (date, entry)
}

// Food database
struct FoodDatabase {
    foods: HashMap<FoodId, Food>,
}

impl FoodDatabase {
    fn new() -> Self {
        FoodDatabase {
            foods: HashMap::new(),
        }
    }

    // For extending to handle an additional website
    fn add_foods_from_source(&mut self, source: &dyn FoodDataSource) {
        let new_foods = source.fetch_food_data();
        for food in new_foods {
            self.add_food(food);
        }
        self.calculate_composite_calories();
    }
    
    fn add_food(&mut self, food: Food) {
        self.foods.insert(food.id.clone(), food);
    }
    
    fn get_food(&self, id: &str) -> Option<&Food> {
        self.foods.get(id)
    }
    
    fn get_foods_by_keywords(&self, keywords: &[String], match_all: bool) -> Vec<&Food> {
        self.foods.values()
            .filter(|food| food.matches_keywords(keywords, match_all))
            .collect()
    }
    
    fn calculate_composite_calories(&mut self) {
        let mut calories_to_update = Vec::new();
        
        for (id, food) in &self.foods {
            if food.is_composite {
                let mut total_calories = 0;
                
                for (component_id, servings) in &food.components {
                    if let Some(component) = self.foods.get(component_id) {
                        total_calories += component.calories_per_serving * servings;
                    }
                }
                
                calories_to_update.push((id.clone(), total_calories));
            }
        }
        
        for (id, calories) in calories_to_update {
            if let Some(food) = self.foods.get_mut(&id) {
                food.calories_per_serving = calories;
            }
        }
    }
    
    fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(e),
        };
        
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() < 4 {
                    continue;
                }
                
                let food_type = parts[0];
                let id = parts[1].to_string();
                let name = parts[2].to_string();
                let keywords: Vec<String> = parts[3].split('|')
                    .map(|s| s.to_string())
                    .collect();
                
                if food_type == "basic" && parts.len() >= 5 {
                    if let Ok(calories) = parts[4].parse::<u32>() {
                        let food = Food::new_basic(&id, &name, keywords, calories);
                        self.add_food(food);
                    }
                } else if food_type == "composite" && parts.len() >= 5 {
                    let components_str = parts[4];
                    let components: Vec<(FoodId, u32)> = components_str
                        .split('|')
                        .filter_map(|comp| {
                            let comp_parts: Vec<&str> = comp.split(':').collect();
                            if comp_parts.len() >= 2 {
                                let food_id = comp_parts[0].to_string();
                                if let Ok(servings) = comp_parts[1].parse::<u32>() {
                                    return Some((food_id, servings));
                                }
                            }
                            None
                        })
                        .collect();
                    
                    let food = Food::new_composite(&id, &name, keywords, components);
                    self.add_food(food);
                }
            }
        }
        
        // Calculate calories for composite foods
        self.calculate_composite_calories();
        
        Ok(())
    }
    
    fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        for food in self.foods.values() {
            let line = food.to_string();
            writeln!(file, "{}", line)?;
        }
        
        Ok(())
    }
}

// Daily log manager
struct DailyLog {
    entries: HashMap<String, Vec<FoodEntry>>, // date -> list of entries
    undo_stack: Vec<CommandType>,
}

impl DailyLog {
    fn new() -> Self {
        DailyLog {
            entries: HashMap::new(),
            undo_stack: Vec::new(),
        }
    }
    
    fn add_food(&mut self, date: &str, food_id: &str, servings: u32) {
        let entry = FoodEntry::new(food_id, servings);
        
        // Store command for undo
        self.undo_stack.push(CommandType::AddFood(date.to_string(), entry.clone()));
        
        // Add to entries
        self.entries
            .entry(date.to_string())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    
    fn delete_food(&mut self, date: &str, index: usize) -> bool {
        if let Some(entries) = self.entries.get_mut(date) {
            if index < entries.len() {
                // Store command for undo
                let entry = entries[index].clone();
                self.undo_stack.push(CommandType::DeleteFood(date.to_string(), entry.clone()));
                
                // Remove the entry
                entries.remove(index);
                return true;
            }
        }
        false
    }
    
    fn undo(&mut self) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            match command {
                CommandType::AddFood(date, _) => {
                    if let Some(entries) = self.entries.get_mut(&date) {
                        if !entries.is_empty() {
                            entries.pop();
                            return true;
                        }
                    }
                },
                CommandType::DeleteFood(date, entry) => {
                    self.entries
                        .entry(date)
                        .or_insert_with(Vec::new)
                        .push(entry);
                    return true;
                }
            }
        }
        false
    }
    
    fn get_entries_for_date(&self, date: &str) -> Vec<&FoodEntry> {
        if let Some(entries) = self.entries.get(date) {
            entries.iter().collect()
        } else {
            Vec::new()
        }
    }
    
    fn calculate_calories_for_date(&self, date: &str, database: &FoodDatabase) -> u32 {
        let mut total_calories = 0;
        
        if let Some(entries) = self.entries.get(date) {
            for entry in entries {
                if let Some(food) = database.get_food(&entry.food_id) {
                    total_calories += food.calories_per_serving * entry.servings;
                }
            }
        }
        
        total_calories
    }
    
    fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(e),
        };
        
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() < 4 { // date,food_id,servings,timestamp
                    continue;
                }
                
                let date = parts[0].to_string();
                if let Some(entry) = FoodEntry::from_string(&parts[1..].join(",")) {
                    self.entries
                        .entry(date)
                        .or_insert_with(Vec::new)
                        .push(entry);
                }
            }
        }
        
        Ok(())
    }
    
    fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        for (date, entries) in &self.entries {
            for entry in entries {
                writeln!(file, "{},{}", date, entry.to_string())?;
            }
        }
        
        Ok(())
    }
}

// User Manager
struct UserManager {
    users: HashMap<String, String>, // username -> password
    data_dir: PathBuf,
}

impl UserManager {
    fn new(data_dir: PathBuf) -> Self {
        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            create_dir_all(&data_dir).expect("Failed to create data directory");
        }
        
        let mut manager = UserManager {
            users: HashMap::new(),
            data_dir,
        };
        
        // Load users from file
        let users_path = manager.data_dir.join("users.txt");
        if users_path.exists() {
            if let Ok(file) = File::open(&users_path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 2 {
                            manager.users.insert(parts[0].to_string(), parts[1].to_string());
                        }
                    }
                }
            }
        }
        
        manager
    }
    
    fn register_user(&mut self, username: &str, password: &str) -> bool {
        if self.users.contains_key(username) {
            return false; // User already exists
        }
        
        self.users.insert(username.to_string(), password.to_string());
        self.save_users();
        
        // Create user directory
        let user_dir = self.data_dir.join(username);
        if !user_dir.exists() {
            create_dir_all(&user_dir).expect("Failed to create user directory");
        }
        
        true
    }
    
    fn authenticate(&self, username: &str, password: &str) -> bool {
        if let Some(stored_password) = self.users.get(username) {
            stored_password == password
        } else {
            false
        }
    }
    
    fn save_users(&self) {
        let users_path = self.data_dir.join("users.txt");
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(users_path) 
        {
            for (username, password) in &self.users {
                if let Err(e) = writeln!(file, "{},{}", username, password) {
                    println!("Error saving users: {}", e);
                }
            }
        }
    }
    
    fn get_user_dir(&self, username: &str) -> PathBuf {
        self.data_dir.join(username)
    }
}

// Main application
struct YadaApplication {
    food_database: FoodDatabase,
    user_profile: Option<UserProfile>,
    daily_log: DailyLog,
    current_date: String,
    running: bool,
    user_manager: UserManager,
    current_user: Option<String>,
    app_undo_stack: Vec<UserProfile>,
}

impl YadaApplication {
    fn new() -> Self {
        // Create data directory
        let data_dir = PathBuf::from("data");
        if !data_dir.exists() {
            create_dir_all(&data_dir).expect("Failed to create data directory");
        }
        
        // Get current date in YYYY-MM-DD format
        let current_date = Self::get_current_date_string();
        
        YadaApplication {
            food_database: FoodDatabase::new(),
            user_profile: None,
            daily_log: DailyLog::new(),
            current_date,
            running: true,
            user_manager: UserManager::new(data_dir),
            current_user: None,
            app_undo_stack: Vec::new(),
        }
    }
    
    // Get current date as YYYY-MM-DD string
    fn get_current_date_string() -> String {
        use chrono::Local;
        Local::now().format("%Y-%m-%d").to_string()
    }
    
    
    // Fixed date conversion that correctly handles days in months and leap years
    fn timestamp_to_date_fixed(timestamp: u64) -> (u32, u32, u32) {
        let secs_per_day = 86400;
        let days_since_epoch = (timestamp / secs_per_day) as i32;
        
        // Starting from 1970-01-01
        let mut year = 1970;
        let mut month = 1;
        let mut day = 1;
        
        // Days in each month (non-leap year)
        let _days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        // Add days to starting date
        let mut days_remaining = days_since_epoch;
        
        while days_remaining > 0 {
            // Check if current year is a leap year
            let leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if leap_year { 366 } else { 365 };
            
            if days_remaining >= days_in_year {
                // Skip the entire year
                days_remaining -= days_in_year;
                year += 1;
            } else {
                // Process day by day within the year
                let feb_days = if leap_year { 29 } else { 28 };
                let month_days = [31, feb_days, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                
                // Find which month
                for (m, &days) in month_days.iter().enumerate() {
                    if days_remaining >= days as i32 {
                        days_remaining -= days as i32;
                        month = m as u32 + 1;
                    } else {
                        day = days_remaining as u32 + 1;
                        days_remaining = 0;
                        break;
                    }
                }
            }
        }
        
        (year as u32, month, day)
    }
    
    fn run(&mut self) {
        println!("Welcome to YADA (Yet Another Diet Assistant)!");
        
        // Start with login or registration
        self.login_or_register();
        
        // Main program loop
        while self.running {
            if self.current_user.is_some() {
                self.display_menu();
                self.process_menu_selection();
            } else {
                // Not logged in, back to login
                self.login_or_register();
            }
        }
        
        println!("Thank you for using YADA. Goodbye!");
    }
    
    fn login_or_register(&mut self) {
        loop {
            println!("\n===== YADA Login =====");
            println!("1. Login");
            println!("2. Register");
            println!("0. Exit");
            
            println!("Enter your choice: ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            
            match input.trim().parse::<u32>() {
                Ok(1) => {
                    if self.login() {
                        // Load user data
                        self.load_user_data();
                        return; // Exit the loop on successful login
                    }
                    // If login fails, continue the loop
                },
                Ok(2) => {
                    if self.register() {
                        // Profile creation is already handled in register()
                        return; // Exit the loop on successful registration
                    }
                    // If registration fails, continue the loop
                },
                Ok(0) => {
                    self.current_user = None;
                    self.running = false; // Set the main program loop to exit
                    return; // Exit this loop immediately
                },
                _ => println!("Invalid option, please try again."),
            }
        }
    }
    
    fn login(&mut self) -> bool {
        println!("Enter username: ");
        let mut username = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        let username = username.trim().to_string();
        
        println!("Enter password: ");
        let mut password = String::new();
        std::io::stdin().read_line(&mut password).unwrap();
        let password = password.trim().to_string();
        
        if self.user_manager.authenticate(&username, &password) {
            println!("Login successful. Welcome, {}!", username);
            self.current_user = Some(username);
            true
        } else {
            println!("Invalid username or password.");
            false
        }
    }
    
    fn register(&mut self) -> bool {
        println!("Enter new username: ");
        let mut username_input = String::new();
        std::io::stdin().read_line(&mut username_input).unwrap();
        let username = username_input.trim().to_string();
        
        if username.is_empty() {
            println!("Username cannot be empty.");
            return false;
        }
        
        println!("Enter password: ");
        let mut password = String::new();
        std::io::stdin().read_line(&mut password).unwrap();
        let password = password.trim().to_string();
        
        if password.is_empty() {
            println!("Password cannot be empty.");
            return false;
        }
        
        if self.user_manager.register_user(&username, &password) {
            println!("Registration successful. Welcome, {}!", username);
            
            // Store username before moving it
            let username_copy = username.clone();
            self.current_user = Some(username);
            
            // First load the food database if it exists
            let db_path = Path::new("data/foods.txt");
            if db_path.exists() {
                if let Err(e) = self.food_database.load_from_file(db_path) {
                    println!("Could not load food database: {}", e);
                    // Only create sample data if database doesn't exist
                    self.create_sample_data();
                    self.save_food_database();
                }
            } else {
                // Create sample data if database doesn't exist
                self.create_sample_data();
                self.save_food_database();
            }
            
            // For new users, we create a profile here
            println!("Creating new profile for {}...", username_copy);
            self.create_user_profile();
            self.daily_log = DailyLog::new();
            self.save_user_data();
            
            true
        } else {
            println!("Username already exists.");
            false
        }
    }
    
    fn load_user_data(&mut self) {
        // We need to clone the username to avoid the borrowing issue
        let username_copy = match &self.current_user {
            Some(name) => name.clone(),
            None => return,
        };
        
        let user_dir = self.user_manager.get_user_dir(&username_copy);
        
        // Load food database (shared among all users)
        let db_path = Path::new("data/foods.txt");
        if let Err(_) = self.food_database.load_from_file(db_path) {
            println!("Could not load food database. Creating sample data...");
            self.create_sample_data();
            if let Err(e) = self.food_database.save_to_file(db_path) {
                println!("Error saving food database: {}", e);
            }
        }
        
        // Load user's daily log
        let log_path = user_dir.join("log.txt");
        self.daily_log = DailyLog::new(); // Reset log before loading
        if log_path.exists() {
            if let Err(e) = self.daily_log.load_from_file(&log_path) {
                println!("Could not load daily log: {}", e);
            }
        }
        
        // Load user profile
        let profile_path = user_dir.join("profile.txt");
        self.user_profile = None; // Reset profile before loading
        
        let profile_exists = profile_path.exists();
        if profile_exists {
            if let Ok(file) = File::open(&profile_path) {
                let mut reader = BufReader::new(file);
                let mut content = String::new();
                
                // Print the raw content for debugging
                if reader.read_line(&mut content).is_ok() {
                    println!("Debug - Profile content: '{}'", content.trim());
                    
                    if !content.trim().is_empty() {
                        if let Some(profile) = UserProfile::from_string(&content.trim()) {
                            self.user_profile = Some(profile);
                            println!("Welcome back, {}!", username_copy);
                        } else {
                            println!("Error parsing profile data. Creating new profile.");
                            // Remove the corrupted profile file
                            if let Err(e) = std::fs::remove_file(&profile_path) {
                                println!("Warning: Could not remove corrupted profile: {}", e);
                            }
                            self.create_user_profile();
                        }
                    } else {
                        println!("Error: Profile file is empty. Creating new profile.");
                        // Remove the empty profile file
                        if let Err(e) = std::fs::remove_file(&profile_path) {
                            println!("Warning: Could not remove empty profile: {}", e);
                        }
                        self.create_user_profile();
                    }
                } else {
                    println!("Error reading profile data. Creating new profile.");
                    self.create_user_profile();
                }
            } else {
                println!("Error opening profile file. Creating new profile.");
                self.create_user_profile();
            }
        } else {
            // Only create a profile for brand new users
            println!("No profile found. Let's create one for you.");
            self.create_user_profile();
        }
    }
    
    fn save_user_data(&self) {
        if let Some(username) = &self.current_user {
            let user_dir = self.user_manager.get_user_dir(username);
            
            // Ensure user directory exists
            if !user_dir.exists() {
                create_dir_all(&user_dir).expect("Failed to create user directory");
            }
            
            // Only save food database if it's a new file, not on every save operation
            let db_path = Path::new("data/foods.txt");
            if !db_path.exists() {
                if let Err(e) = self.food_database.save_to_file(db_path) {
                    println!("Error saving food database: {}", e);
                }
            }
            
            // Save user's daily log
            let log_path = user_dir.join("log.txt");
            if let Err(e) = self.daily_log.save_to_file(&log_path) {
                println!("Error saving daily log: {}", e);
            }
            
            // Save user profile
            if let Some(profile) = &self.user_profile {
                let profile_path = user_dir.join("profile.txt");
                if let Ok(mut file) = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(profile_path) 
                {
                    if let Err(e) = writeln!(file, "{}", profile.to_string()) {
                        println!("Error saving user profile: {}", e);
                    }
                }
            }
        }
    }

    fn create_sample_data(&mut self) {
        // Add basic foods
        let basic_foods = vec![
            Food::new_basic("chicken", "Chicken Breast", vec!["chicken".to_string(), "meat".to_string(), "protein".to_string()], 165),
            Food::new_basic("apple", "Apple", vec!["apple".to_string(), "fruit".to_string()], 95),
            Food::new_basic("pb", "Peanut Butter", vec!["peanut".to_string(), "butter".to_string()], 190),
            Food::new_basic("rice", "White Rice", vec!["rice".to_string(), "grain".to_string()], 206),
            Food::new_basic("butter", "Butter", vec!["butter".to_string(), "fat".to_string()], 102),
            Food::new_basic("bread", "Bread Slice", vec!["bread".to_string(), "grain".to_string()], 80),
            Food::new_basic("egg", "Egg", vec!["egg".to_string(), "protein".to_string()], 78),
            Food::new_basic("banana", "Banana", vec!["banana".to_string(), "fruit".to_string()], 105),
            Food::new_basic("seeds", "seeds", vec!["seed".to_string(), "seeds".to_string()], 300),
            Food::new_basic("milk", "Whole Milk", vec!["milk".to_string(), "dairy".to_string()], 149),
            Food::new_basic("sprout", "sprouts", vec!["sprout".to_string(), "sprouts".to_string()], 250),
            Food::new_basic("cheese", "Cheddar Cheese", vec!["cheese".to_string(), "dairy".to_string()], 113),
        ];
        
        for food in basic_foods {
            self.food_database.add_food(food);
            // self.food_database.add_foods_from_source(&dummy_source);
        }
        
        // Add composite foods
        let composite_foods = vec![
            ("pb_sandwich", "Peanut Butter Sandwich", vec!["sandwich".to_string(), "peanut".to_string()], 
             vec![("bread".to_string(), 2), ("pb".to_string(), 1)]),
            ("csalad", "chicken salad", vec!["salad".to_string(), "chicken".to_string(), "greens".to_string()],
             vec![("chicken".to_string(), 2), ("sprout".to_string(), 1)]),
            ("vada", "medhu vada", vec!["medhu".to_string(), "vada".to_string()],
             vec![("rice".to_string(), 2), ("sprout".to_string(), 2)]),
            ("bshake", "banana shake", vec!["shake".to_string(), "banana".to_string(), "bananashake".to_string()],
             vec![("banana".to_string(), 2), ("milk".to_string(), 2)])
        ];
        
        for (id, name, keywords, components) in composite_foods {
            let food = Food::new_composite(id, name, keywords, components);
            self.food_database.add_food(food);
        }
        
        // Calculate calories for composite foods
        self.food_database.calculate_composite_calories();
    }

    fn save_food_database(&self) {
        let db_path = Path::new("data/foods.txt");
        if let Err(e) = self.food_database.save_to_file(db_path) {
            println!("Error saving food database: {}", e);
        }
    }
    
    fn create_user_profile(&mut self) {
        println!("\nLet's set up your profile:");
        
        let username = match &self.current_user {
            Some(name) => name.clone(),
            None => {
                println!("Error: No user logged in.");
                return;
            }
        };
        
        println!("Enter your gender (M/F/O): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let gender = match input.trim().to_lowercase().as_str() {
            "m" | "male" => Gender::Male,
            "f" | "female" => Gender::Female,
            _ => Gender::Other,
        };
        
        println!("Enter your height (cm): ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let height = input.trim().parse::<f64>().unwrap_or(170.0);
        
        println!("Enter your age: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let age = input.trim().parse::<u32>().unwrap_or(30);
        
        println!("Enter your weight (kg): ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let weight = input.trim().parse::<f64>().unwrap_or(70.0);
        
        println!("Enter your activity level (1-5):");
        println!("1. Sedentary");
        println!("2. Lightly Active");
        println!("3. Moderately Active");
        println!("4. Very Active");
        println!("5. Extremely Active");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let activity_level = match input.trim().parse::<u32>().unwrap_or(3) {
            1 => ActivityLevel::Sedentary,
            2 => ActivityLevel::LightlyActive,
            3 => ActivityLevel::ModeratelyActive,
            4 => ActivityLevel::VeryActive,
            5 => ActivityLevel::ExtremelyActive,
            _ => ActivityLevel::ModeratelyActive,
        };
        
        self.user_profile = Some(UserProfile::new(username, gender, height, age, weight, activity_level));
        
        println!("Profile created successfully!\n");
    }
    
    fn display_menu(&self) {
        println!("\n===== YADA DIET MANAGER =====");
        if let Some(username) = &self.current_user {
            println!("Logged in as: {}", username);
        }
        println!("Current Date: {}", self.current_date);
        
        if let Some(profile) = &self.user_profile {
            let target_calories = profile.get_target_calories();
            let consumed_calories = self.daily_log.calculate_calories_for_date(&self.current_date, &self.food_database);
            let diff = consumed_calories as i32 - target_calories as i32;  // raw difference
        
            println!("Target Calories: {}", target_calories);
            println!("Consumed Calories: {}", consumed_calories);
            println!("Difference (consumed - target): {}", diff);
            if diff < 0 {
                println!("(Negative: {} calories available)", diff);
            } else if diff > 0 {
                println!("(Positive: {} calories consumed in excess)", diff);
            } else {
                println!("(Exactly met the target!)");
            }
        }
        
        
        println!("\nMenu Options:");
        println!("1. Add Food to Log");
        println!("2. View Today's Log");
        println!("3. Delete Food from Log");
        println!("4. Undo Last Action");
        println!("5. Change Date");
        println!("6. Add Basic Food to Database");
        println!("7. Create Composite Food");
        println!("8. Update Profile");
        println!("9. Change Calorie Calculation Method");
        println!("10. Save Data");
        println!("11. Logout");
        println!("0. Exit");
        
        print!("Enter your choice: ");
    }    
    
    fn process_menu_selection(&mut self) {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().parse::<u32>() {
            Ok(1) => self.add_food_to_log(),
            Ok(2) => self.view_log(),
            Ok(3) => self.delete_food_from_log(),
            Ok(4) => self.undo_action(),
            Ok(5) => self.change_date(),
            Ok(6) => self.add_basic_food(),
            Ok(7) => self.create_composite_food(),
            Ok(8) => self.update_profile(),
            Ok(9) => self.change_calculation_method(),
            Ok(10) => {
                self.save_user_data();
                println!("Data saved successfully.");
            },
            Ok(11) => {
                self.save_user_data();
                self.current_user = None;
                println!("Logged out successfully.");
            },
            Ok(0) => {
                self.save_user_data();
                self.running = false;
            },
            _ => println!("Invalid option, please try again."),
        }
    }
    
    fn add_food_to_log(&mut self) {
        println!("\nAdd Food to Log");
        println!("1. Search by keyword");
        println!("2. List all foods");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let foods = match input.trim().parse::<u32>() {
            Ok(1) => {
                println!("Enter keywords (space separated): ");
                input.clear();
                std::io::stdin().read_line(&mut input).unwrap();
                let keywords: Vec<String> = input.trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                
                println!("Match all keywords? (y/n): ");
                input.clear();
                std::io::stdin().read_line(&mut input).unwrap();
                let match_all = input.trim().to_lowercase().starts_with('y');
                
                self.food_database.get_foods_by_keywords(&keywords, match_all)
            },
            Ok(2) => {
                self.food_database.foods.values().collect()
            },
            _ => {
                println!("Invalid option.");
                return;
            }
        };
        
        if foods.is_empty() {
            println!("No foods found matching your criteria.");
            return;
        }
        
        println!("\nAvailable Foods:");
        for (i, food) in foods.iter().enumerate() {
            println!("{}. {} ({} calories/serving)", i + 1, food.name, food.calories_per_serving);
        }
        
        println!("\nSelect a food (enter number): ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let selection = match input.trim().parse::<usize>() {
            Ok(n) if n > 0 && n <= foods.len() => n - 1,
            _ => {
                println!("Invalid selection.");
                return;
            }
        };
        
        println!("Enter number of servings: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let servings = match input.trim().parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => {
                println!("Invalid number of servings.");
                return;
            }
        };
        
        let selected_food = foods[selection];
        self.daily_log.add_food(&self.current_date, &selected_food.id, servings);
        println!("Added {} serving(s) of {} to log.", servings, selected_food.name);
    }
    
    fn view_log(&self) {
        println!("\nFood Log for {}", self.current_date);
        
        let entries = self.daily_log.get_entries_for_date(&self.current_date);
        
        if entries.is_empty() {
            println!("No entries found for this date.");
            return;
        }
        
        println!("ID | Food | Servings | Calories");
        println!("---------------------------------");
        
        let mut total_calories = 0;
        
        for (i, entry) in entries.iter().enumerate() {
            if let Some(food) = self.food_database.get_food(&entry.food_id) {
                let calories = food.calories_per_serving * entry.servings;
                total_calories += calories;
                println!("{}. {} | {} | {} cal", 
                         i + 1, food.name, entry.servings, calories);
            }
        }
        
        println!("---------------------------------");
        println!("Total Calories: {}", total_calories);
        
        if let Some(profile) = &self.user_profile {
            let target = profile.get_target_calories();
            let diff = self.daily_log.calculate_calories_for_date(&self.current_date, &self.food_database) as i32 - target as i32;
        
            println!("Target Calories: {}", target);
            println!("Difference (consumed - target): {}", diff);
            if diff < 0 {
                println!("Negative value indicates calories available.");
            } else if diff > 0 {
                println!("Positive value indicates consumption in excess.");
            } else {
                println!("Exact match with the target.");
            }
        }        
    }
    
    fn delete_food_from_log(&mut self) {
        println!("\nDelete Food from Log");
        
        let entries = self.daily_log.get_entries_for_date(&self.current_date);
        
        if entries.is_empty() {
            println!("No entries found for this date.");
            return;
        }
        
        println!("Current Entries:");
        for (i, entry) in entries.iter().enumerate() {
            if let Some(food) = self.food_database.get_food(&entry.food_id) {
                println!("{}. {} ({} servings)", i + 1, food.name, entry.servings);
            }
        }
        
        println!("\nEnter the number of the entry to delete: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().parse::<usize>() {
            Ok(n) if n > 0 && n <= entries.len() => {
                if self.daily_log.delete_food(&self.current_date, n - 1) {
                    println!("Entry deleted successfully.");
                } else {
                    println!("Failed to delete entry.");
                }
            },
            _ => println!("Invalid selection."),
        }
    }
    
    fn undo_action(&mut self) {
        println!("Choose undo type:");
        println!("1. Daily Log Action");
        println!("2. Profile Update");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        match input.trim().parse::<u32>() {
            Ok(1) => {
                if self.daily_log.undo() {
                    println!("Last log action undone successfully.");
                } else {
                    println!("Nothing to undo in the daily log.");
                }
            },
            Ok(2) => {
                if let Some(old_profile) = self.app_undo_stack.pop() {
                    self.user_profile = Some(old_profile);
                    println!("Profile update undone successfully.");
                } else {
                    println!("Nothing to undo in profile updates.");
                }
            },
            _ => println!("Invalid selection for undo."),
        }
    }
    
    
    fn change_date(&mut self) {
        println!("\nChange Date");
        println!("Enter date (YYYY-MM-DD): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let date = input.trim();
        // Simple validation - more sophisticated validation would be better
        if date.len() == 10 && date.chars().nth(4) == Some('-') && date.chars().nth(7) == Some('-') {
            self.current_date = date.to_string();
            println!("Date changed to {}", self.current_date);
        } else {
            println!("Invalid date format. Please use YYYY-MM-DD.");
        }
    }
    
    fn add_basic_food(&mut self) {
        println!("\nAdd Basic Food to Database");
        
        println!("Enter food ID (unique identifier, no spaces): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let id = input.trim().to_string();
        
        // Check if ID already exists
        if self.food_database.get_food(&id).is_some() {
            println!("A food with ID '{}' already exists.", id);
            return;
        }
        
        println!("Enter food name: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let name = input.trim().to_string();
        
        println!("Enter keywords (space separated): ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let keywords: Vec<String> = input.trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        println!("Enter calories per serving: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let calories = match input.trim().parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => {
                println!("Invalid calories value.");
                return;
            }
        };
        
        let food = Food::new_basic(&id, &name, keywords, calories);
        self.food_database.add_food(food);
        
        // Save food database after adding a new food
        self.save_food_database();
        
        println!("Food '{}' added successfully to the database.", name);
    }
    
    fn create_composite_food(&mut self) {
        println!("\nCreate Composite Food");
        
        println!("Enter composite food ID (unique identifier, no spaces): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let id = input.trim().to_string();
        
        // Check if ID already exists
        if self.food_database.get_food(&id).is_some() {
            println!("A food with ID '{}' already exists.", id);
            return;
        }
        
        println!("Enter food name: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let name = input.trim().to_string();
        
        println!("Enter keywords (space separated): ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let keywords: Vec<String> = input.trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        let mut components: Vec<(FoodId, u32)> = Vec::new();
        
        loop {
            println!("\nAdd Components (enter 0 to finish):");
            println!("1. Search for component by keyword");
            println!("2. List all available foods");
            println!("0. Finish adding components");
            
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();
            
            match input.trim().parse::<u32>() {
                Ok(0) => break,
                Ok(1) => {
                    println!("Enter keywords (space separated): ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let search_keywords: Vec<String> = input.trim()
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    
                    println!("Match all keywords? (y/n): ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let match_all = input.trim().to_lowercase().starts_with('y');
                    
                    let foods = self.food_database.get_foods_by_keywords(&search_keywords, match_all);
                    self.select_and_add_component(&foods, &mut components);
                },
                Ok(2) => {
                    let foods: Vec<&Food> = self.food_database.foods.values().collect();
                    self.select_and_add_component(&foods, &mut components);
                },
                _ => println!("Invalid option."),
            }
        }
        
        if components.is_empty() {
            println!("No components added. Composite food creation cancelled.");
            return;
        }
        
        let food = Food::new_composite(&id, &name, keywords, components);
        self.food_database.add_food(food);
        
        // Recalculate calories for all composite foods
        self.food_database.calculate_composite_calories();
        
        // Save food database after adding a new composite food
        self.save_food_database();
        
        println!("Composite food '{}' created successfully.", name);
    }
    
    fn select_and_add_component(&self, foods: &[&Food], components: &mut Vec<(FoodId, u32)>) {
        if foods.is_empty() {
            println!("No foods found matching your criteria.");
            return;
        }
        
        println!("\nAvailable Foods:");
        for (i, food) in foods.iter().enumerate() {
            println!("{}. {} ({} calories/serving)", i + 1, food.name, food.calories_per_serving);
        }
        
        println!("\nSelect a food (enter number): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let selection = match input.trim().parse::<usize>() {
            Ok(n) if n > 0 && n <= foods.len() => n - 1,
            _ => {
                println!("Invalid selection.");
                return;
            }
        };
        
        println!("Enter number of servings: ");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let servings = match input.trim().parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => {
                println!("Invalid number of servings.");
                return;
            }
        };
        
        let selected_food = foods[selection];
        components.push((selected_food.id.clone(), servings));
        
        println!("Added {} serving(s) of {} as a component.", 
                 servings, selected_food.name);
    }
    
    fn update_profile(&mut self) {
        if let Some(profile) = &mut self.user_profile {

            self.app_undo_stack.push(profile.clone());

            println!("\n===== Update Profile =====");
            println!("Current Profile:");
            println!("Username: {}", profile.username);
            println!("Gender: {:?}", profile.gender);
            println!("Height: {} cm", profile.height);
            println!("Age: {}", profile.age);
            println!("Weight: {} kg", profile.weight);
            println!("Activity Level: {:?}", profile.activity_level);
            
            println!("\nWhat would you like to update?");
            println!("1. Weight");
            println!("2. Age");
            println!("3. Height");
            println!("4. Gender");
            println!("5. Activity Level");
            println!("0. Cancel");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            
            match input.trim().parse::<u32>() {
                Ok(1) => {
                    println!("Enter new weight (kg): ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if let Ok(weight) = input.trim().parse::<f64>() {
                        profile.weight = weight;
                        println!("Weight updated to {} kg.", weight);
                    } else {
                        println!("Invalid weight value.");
                    }
                },
                Ok(2) => {
                    println!("Enter new age: ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if let Ok(age) = input.trim().parse::<u32>() {
                        profile.age = age;
                        println!("Age updated to {}.", age);
                    } else {
                        println!("Invalid age value.");
                    }
                },
                Ok(3) => {
                    println!("Enter new height (cm): ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if let Ok(height) = input.trim().parse::<f64>() {
                        profile.height = height;
                        println!("Height updated to {} cm.", height);
                    } else {
                        println!("Invalid height value.");
                    }
                },
                Ok(4) => {
                    println!("Enter gender (M/F/O): ");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let gender = match input.trim().to_lowercase().as_str() {
                        "m" | "male" => Gender::Male,
                        "f" | "female" => Gender::Female,
                        _ => Gender::Other,
                    };
                    profile.gender = gender;
                    println!("Gender updated to {:?}.", gender);
                },
                Ok(5) => {
                    println!("Enter activity level (1-5):");
                    println!("1. Sedentary");
                    println!("2. Lightly Active");
                    println!("3. Moderately Active");
                    println!("4. Very Active");
                    println!("5. Extremely Active");
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let activity_level = match input.trim().parse::<u32>().unwrap_or(3) {
                        1 => ActivityLevel::Sedentary,
                        2 => ActivityLevel::LightlyActive,
                        3 => ActivityLevel::ModeratelyActive,
                        4 => ActivityLevel::VeryActive,
                        5 => ActivityLevel::ExtremelyActive,
                        _ => ActivityLevel::ModeratelyActive,
                    };
                    profile.activity_level = activity_level;
                    println!("Activity level updated.");
                },
                Ok(0) => println!("Update cancelled."),
                _ => println!("Invalid option."),
            }
        } else {
            println!("No profile exists. Please create one first.");
            self.create_user_profile();
        }
    }
    
    fn change_calculation_method(&mut self) {
        if let Some(profile) = &mut self.user_profile {
            println!("\nChange Calorie Calculation Method");
            println!("Current method: {}", profile.calculation_method);
            println!("Available methods:");
            println!("1. Harris-Benedict Equation");
            println!("2. Mifflin-St Jeor Equation");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            
            match input.trim().parse::<u32>() {
                Ok(1) => {
                    profile.set_calculation_method("harris-benedict");
                    println!("Calculation method changed to Harris-Benedict Equation.");
                },
                Ok(2) => {
                    profile.set_calculation_method("mifflin-st-jeor");
                    println!("Calculation method changed to Mifflin-St Jeor Equation.");
                },
                _ => println!("Invalid option."),
            }
        } else {
            println!("No profile exists. Please create one first.");
            self.create_user_profile();
        }
    }
}

fn main() {
    let mut app = YadaApplication::new();
    app.run();
}