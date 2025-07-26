def calculate_sum(a, b):
    return a + b

def main():
    result = calculate_sum(1, 2, 3)  # Too many arguments
    print result  # Missing parentheses (Python 3 syntax error)
    return result

if __name__ == "__main__":
    main()
