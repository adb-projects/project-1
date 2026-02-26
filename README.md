# Multi-Provider Data Normalization Service
## Setup & Running
### Dependencies installation

The application is built in rust and a rust dev environment is required to build and run

### How to run the application
1. Run the executable
```
cargo run
```
2. The executable will create two directories 
    * input - add files (example files are in test-data directory)
    * normalized - output of input files
3. files found in the input folder will be processed and put into the normalized folder

### How to run tests
```
cargo test
```
## Design Decisions
Each file type is processed by a file provider that understands it schema and converts the contents to the normalized format.  Additional file providers built for different schemas and added to the handle_data method in main.rs.
