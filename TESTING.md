# Multi-Provider Data Normalization Service
## Testing Strategy
### What I Tested
- Tested the inputs for all three files types using unit tests
### What I'd Test With More Time (Prioritized)
1. Testing of edge cases needs to be added (also better error handling)
2. Test the logic in main.rs (not related to the parsing logic but needs test coverage)
### How I Made This Testable
The seperate providers enables the logic for the different file types to be tested seperately