# YADA Diet Manager

## Overview
YADA (Yet Another Diet Assistant) is a prototype diet management system implemented in Rust. The application is designed to help users track their daily food consumption, manage a food database (with support for both basic and composite foods), and monitor calorie goals based on personal profile data. The system is built as a command‑line interface (CLI) application and emphasizes extensibility and efficient data management.

## Features

- **User Management**
  - **Registration & Login:** Users can register a new account and then log in using their username and password.
  - **Per‑User Data Storage:** Each user’s profile, daily log, and related data are stored in separate directories under the `data/` folder.
  
- **Food Database**
  - **Basic Foods:** Store and manage foods, each defined by an identifier, a list of search keywords, and calories per serving.
  - **Composite Foods:** Create composite foods by combining basic (or other composite) foods with a specified serving count. The calorie count for a composite is calculated as the sum of its components.
  - **Database Persistence:** Food data is maintained in a human‑readable text file (`data/foods.txt`), which is loaded at startup and can be saved at any time via the “Save Data” option.
  - **Extensible Data Sources:** The design includes a `FoodDataSource` trait and a dummy implementation as proof-of-concept for easily integrating web data.

- **Daily Logs**
  - **Add Food Entries:** Users can add food entries to their daily log by searching foods with keywords or listing all available foods.
  - **Delete and Update Entries:** Users can remove entries (allowing them to adjust serving counts) and view the full log.
  - **Undo Functionality:** Undo any previous food addition or deletion, with no fixed limit (except by available memory). In addition, users can undo profile updates separately.
  - **Date Management:** Users can change the active log date to view and edit past or future logs.

- **Diet Goal Profile**
  - **Profile Settings:** Record the user’s gender, height, age, weight, and activity level. The system carries over daily values by default.
  - **Calorie Computation:** Compute target calorie intake using at least two methods (Harris-Benedict and Mifflin-St Jeor) and switch between them on demand.
  - **Calorie Tracking:** At any point, display the total calories consumed, target calorie intake, and the raw difference (with negative values indicating calories available and positive values representing excess).

- **Extensibility and Efficiency**
  - **Modular Design:** By using traits such as `FoodDataSource` and `CalorieCalculator`, the design facilitates future extensions with minimal code changes.
  - **Optimized Logs:** Food entries store only identifiers (rather than full food details), reducing redundancy in the log file as it grows.

Below is the new Assumptions section that you can add to your README file:

---

## Assumptions

- Every time a user registers or adds a food item to the log, it is treated as a distinct meal entry and saved as a new record in the log, not a duplicate record (since we can differentiate based on timestamp).
- The food identifier (ID) is unique; duplicate IDs for basic or composite foods are not allowed.
- User data (profiles and logs) and the food database are stored and managed via human‑readable text files, and the application assumes that these files are accessible and writable.
- Composite foods are created by combining existing basic or composite foods; their calorie counts are calculated at the time of creation based solely on the component foods’ defined calories.
- The undo functionality is session‑specific; all undo information is discarded when the program terminates.
- When switching dates in the log, the input date is assumed to be in the correct format (YYYY‑MM‑DD).
- The system currently carries over profile data from one day to the next unless explicitly updated by the user.
- The design assumes that the initial food database and user file structures (e.g., folder structure under the `data/` directory) are correctly set up before running the application.

---

## Requirements

- **Rust Programming Environment:**  
  Ensure that you have the latest Rust compiler installed. You can install Rust from [rustup.rs](https://rustup.rs/).

- **Cargo:**  
  This project uses Cargo for dependency management and building. The `Cargo.toml` includes a dependency on the Chrono crate (`chrono = "0.4"`).

## How to Run

1. **Clone or Download the Project:**
   - Place the project files in a directory on your computer.
   
2. **Compile the Project:**
   - Open a terminal in the project directory and run:
     ```
     cargo build
     ```
   - This will download dependencies and compile the project.

3. **Run the Application:**
   - To run the application, execute:
     ```
     cargo run
     ```
   - The CLI will start with a welcome message and show the initial login/registration screen.

## Using the Application

### Registration and Login

- **Registration:**
  - When prompted, choose the "Register" option.
  - Enter a new username and password.
  - You will then be guided through the profile creation process (entering gender, height, age, weight, and activity level).

- **Login:**
  - If you already have an account, choose the "Login" option.
  - Enter your username and password.
  - Upon successful authentication, your user-specific data (profile and log) will be loaded.

### Main Menu Options

After logging in, the main menu displays current date, calorie information, and various options:

1. **Add Food to Log:**
   - Option 1 allows you to add a food entry.
   - Choose between searching by keyword or listing all foods.
   - Input the number of servings to add to your log.

2. **View Today's Log:**
   - Option 2 shows all food entries for the current date.
   - Displays food names, servings, calculated calories per entry, total consumed calories, target calories, and the raw difference (consumed – target).

3. **Delete Food from Log:**
   - Option 3 lets you remove a food entry by its list number.
   - Useful if you need to adjust servings (by deleting then re-adding).

4. **Undo Last Action:**
   - Option 4 provides two types:
     - Daily Log Action: Undo the last addition or deletion in your log.
     - Profile Update: Revert the most recent change to your profile.

5. **Change Date:**
   - Option 5 allows you to set a different date (format: YYYY-MM-DD) to view or modify logs for other days.

6. **Add Basic Food to Database:**
   - Option 6 lets you add a new basic food.
   - Input the food’s unique identifier, name, keywords, and calories per serving.
   
7. **Create Composite Food:**
   - Option 7 guides you through creating a composite food.
   - Enter a unique identifier, name, and keywords, then select component foods (with serving counts) from available lists.
   - The system will compute the total calories automatically.

8. **Update Profile:**
   - Option 8 enables you to update your profile fields (e.g., weight, age, height, gender, activity level).
   - The previous profile can be undone through the undo option if needed.

9. **Change Calorie Calculation Method:**
   - Option 9 allows you to toggle between the Harris-Benedict and Mifflin-St Jeor equations.
   - The target calorie intake is updated accordingly.

10. **Save Data:**
    - Option 10 saves user data, including the food database, daily log, and profile.  
    - Although data is autosaved upon exit, use this option to ensure your session is recorded at any time.

11. **Logout:**
    - Option 11 logs you out of the current session and returns you to the login/registration screen.

0. **Exit:**
   - Option 0 saves your data and terminates the program.

## Testing All Features

- **User Functions:** Test by registering multiple users and logging in/out.
- **Food Database:**  
  - Add a new basic food via option 6.
  - Create a composite food via option 7.
- **Daily Log:**  
  - Add and delete entries using options 1 and 3.
  - Verify undo functionality for both log actions and profile updates via option 4.
  - Change the log date (option 5) to add/view past or future entries.
- **Profile and Calorie Calculations:**  
  - Update your profile and switch calculation methods with options 8 and 9.
  - Observe how the target, consumed, and difference values in the menu and log view update accordingly.
- **Data Persistence:**  
  - Use option 10 to save data.
  - Exit the program and re-run to ensure that logs, profile, and database data are loaded correctly.

## Directory Structure

- **src/main.rs:** Contains the main program logic.
- **Cargo.toml:** Manages project dependencies.
- **data/**  
  - Contains subdirectories for each registered user with their logs and profiles.
  - **foods.txt:** The food database file.
  - **users.txt:** User credentials.

## Final Notes

- This prototype is designed for extensibility. For example, additional food data sources can be integrated by implementing the `FoodDataSource` trait.
- The modular approach for calorie calculations via the `CalorieCalculator` trait allows new formulas to be added with minimal changes.
- Future enhancements might include more robust input parsing, comprehensive error handling, and a more sophisticated logging mechanism for very large log files.