interface Greeting {
    message: string;
    timestamp: Date;
}

function createGreeting(name: string): Greeting {
    return {
        message: 42,  // Type error: number is not assignable to string
        timestamp: "not a date"  // Type error: string is not assignable to Date
    };
}

const greeting = createGreeting(123);  // Type error: number is not assignable to string
console.log(greeting.message)  // Missing semicolon
