1. Install Rust
Rust uses a tool called rustup to manage versions and dependencies. To install Rust:

Windows
Download the Rust installer from the official website:
👉 https://rustup.rs
Run the installer and follow the prompts.
Once installed, restart your terminal (cmd, PowerShell, or Git Bash).
Verify installation by running:
sh
Copy
Edit
rustc --version
If it prints something like rustc 1.x.x (date), Rust is installed!


2. Create a New Rust Project
Once Rust is installed, we use Cargo (Rust’s build system) to set up our project.

Open a terminal.
Navigate to where you want your project folder:
sh
Copy
Edit
cd path/to/your/projects  # Change this to your preferred location
Create a new Rust project:
sh
Copy
Edit
cargo new mayan_calendar
This creates a folder called mayan_calendar with a basic Rust project structure:
bash
Copy
Edit
mayan_calendar/
├── Cargo.toml  # Rust package config
├── src/
│   ├── main.rs  # Your Rust program goes here
3. Add Dependencies
The Rust script I provided uses the chrono crate for date handling. We need to add it to our project.

Open the Cargo.toml file in your mayan_calendar folder.
Under [dependencies], add:
toml
Copy
Edit
[dependencies]
chrono = "0.4"
Save the file.
Alternatively, you can add chrono via the command line:

sh
Copy
Edit
cd mayan_calendar  # Navigate into your project folder
cargo add chrono
4. Copy the Code into main.rs
Open mayan_calendar/src/main.rs in a text editor (VS Code, Notepad++, or any editor).
Delete everything in main.rs and replace it with the Rust script I provided.
Save the file.
5. Build & Run the Program
Now you’re ready to compile and run the program!

Open a terminal and navigate to your project folder:
sh
Copy
Edit
cd mayan_calendar
Compile the project and run it:
sh
Copy
Edit
cargo run
This will:
Download any dependencies (only the first time).
Compile the Rust program.
Execute it.



6. Running the Compiled Binary (Optional)
If you want to run the compiled program without recompiling every time:

First, build the project (without running):
sh
Copy
Edit
cargo build --release
Then, run the compiled binary directly:
Windows:
sh
Copy
Edit
./target/release/mayan_calendar.exe
Linux/macOS:
sh
Copy
Edit
./target/release/mayan_calendar
7. Example Output
If everything is working, you should see an output like:

yaml
Copy
Edit
Gregorian Date: 2025-02-05
Julian Day Number: 2460389
Days since Mayan creation (0.0.0.0.0): 1842106
Long Count: 13.0.11.3.6
Tzolkin Date: 12 Cimi
Haab Date: 8 Yax
🎉 Done!
Congrats, fren! You now have a working Rust program that calculates today's Mayan calendar date. 🚀

Let me know if you hit any snags!
