using System;

// Missing namespace
public class bad_class // Incorrect naming convention
{
    // Public field instead of property
    public string name;
    
    // Missing XML documentation
    public int get_age() { // Incorrect naming convention
        return age; // Reference to non-existent field
    }
    
    // Constructor with invalid parameter name
    public bad_class(string n)
    {
        name = n;
        Console.WriteLine("Created instance")
        // Missing semicolon
    }
    
    // Incomplete method
    public void incomplete_method() {
        if (name == "Test") {
            // Missing closing bracket
    }
}

