using System;

namespace Synx.Test
{
    /// <summary>
    /// A simple class representing a person.
    /// </summary>
    public class Person
    {
        /// <summary>
        /// Gets or sets the person's name.
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Gets or sets the person's age.
        /// </summary>
        public int Age { get; set; }

        /// <summary>
        /// Initializes a new instance of the <see cref="Person"/> class.
        /// </summary>
        /// <param name="name">The person's name.</param>
        /// <param name="age">The person's age.</param>
        public Person(string name, int age)
        {
            Name = name;
            Age = age;
        }

        /// <summary>
        /// Returns a greeting message.
        /// </summary>
        /// <returns>A greeting string.</returns>
        public string Greet()
        {
            return $"Hello, my name is {Name} and I am {Age} years old.";
        }
    }
}

