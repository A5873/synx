/**
 * An invalid JSX file with syntax errors
 * This file should fail validation with Synx
 */
import React from 'react'  // Missing semicolon

// Missing import for useState
// Should be: import { useState } from 'react';

// Unclosed JSX tag
function Header() {
  return (
    <header className="app-header">
      <h1>Task Tracker<h1>  // Unclosed h1 tag
      <nav>
        <ul>
          <li><a href="#home">Home</a></li>
          <li><a href="#tasks">Tasks</a><li>  // Incorrect closing tag
        </ul>
      </nav>
    </header>
  );
}

// Invalid JSX syntax - missing closing parenthesis
const TaskItem = ({ task, onDelete }) => {
  // Missing closing bracket in JSX
  return (
    <div className="task-item">
      <h3>{task.title}</h3
      <p>{task.description}</p>
      <button onClick={() => onDelete(task.id)}>Delete</button>
    </div>
  );
};

// Invalid hook usage - hooks cannot be conditional
function ConditionalHook({ condition }) {
  if (condition) {
    const [state, setState] = useState(0);  // Hook inside condition
  }
  
  return <div>{state}</div>;  // Undefined variable
}

// Missing return statement in component
function EmptyComponent() {
  const items = [1, 2, 3];
  // No return statement
}

class TaskList extends React.Component {
  // Invalid class property syntax
  state = {
    tasks: [
      { id: 1, title: 'Learn React', completed: false,  // Missing closing brace
    ]
  };

  // Invalid event handler - missing function body
  handleClick = (id) => 
  
  render() {
    return (
      <div>
        {/* JSX comment with unclosed brace */}
        {/* This is a problematic comment
        
        {/* Incorrect boolean attribute */}
        <input type="checkbox" checked="true" />
        
        {/* Invalid prop spread syntax */}
        <TaskItem ...{this.props.task} />
        
        {/* Invalid JSX expression - missing closing brace */}
        <p>{this.state.tasks.length} total tasks</p
      </div>
    );
  }

// Missing closing brace for class

export default TaskList;

