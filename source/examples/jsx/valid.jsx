/**
 * A valid JSX file showcasing React component patterns
 * This file should pass validation with Synx
 */
import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';

// Functional component with hooks
const TaskList = ({ initialTasks = [] }) => {
  // State management with hooks
  const [tasks, setTasks] = useState(initialTasks);
  const [newTaskText, setNewTaskText] = useState('');
  const [filter, setFilter] = useState('all');

  // Side effect with useEffect
  useEffect(() => {
    document.title = `${tasks.length} tasks`;
    
    // Cleanup function
    return () => {
      document.title = 'React App';
    };
  }, [tasks]);

  // Event handlers
  const handleAddTask = () => {
    if (newTaskText.trim() === '') return;
    
    const newTask = {
      id: Date.now(),
      text: newTaskText,
      completed: false
    };
    
    setTasks([...tasks, newTask]);
    setNewTaskText('');
  };

  const handleToggleComplete = (taskId) => {
    setTasks(tasks.map(task => 
      task.id === taskId 
        ? { ...task, completed: !task.completed } 
        : task
    ));
  };

  const handleDeleteTask = (taskId) => {
    setTasks(tasks.filter(task => task.id !== taskId));
  };

  // Computed values
  const filteredTasks = tasks.filter(task => {
    if (filter === 'active') return !task.completed;
    if (filter === 'completed') return task.completed;
    return true; // 'all' filter
  });

  // JSX rendering with fragments, conditional rendering, and event handlers
  return (
    <>
      <h1 className="task-header">Task Manager</h1>
      
      {/* Task input form */}
      <div className="task-form">
        <input
          type="text"
          value={newTaskText}
          onChange={(e) => setNewTaskText(e.target.value)}
          placeholder="Add a new task"
        />
        <button onClick={handleAddTask}>Add Task</button>
      </div>
      
      {/* Filter controls */}
      <div className="filter-controls">
        <button 
          className={filter === 'all' ? 'active' : ''}
          onClick={() => setFilter('all')}
        >
          All
        </button>
        <button 
          className={filter === 'active' ? 'active' : ''}
          onClick={() => setFilter('active')}
        >
          Active
        </button>
        <button 
          className={filter === 'completed' ? 'active' : ''}
          onClick={() => setFilter('completed')}
        >
          Completed
        </button>
      </div>
      
      {/* Task list */}
      {filteredTasks.length > 0 ? (
        <ul className="task-list">
          {filteredTasks.map(task => (
            <li key={task.id} className={task.completed ? 'completed' : ''}>
              <input
                type="checkbox"
                checked={task.completed}
                onChange={() => handleToggleComplete(task.id)}
              />
              <span className="task-text">{task.text}</span>
              <button 
                className="delete-btn"
                onClick={() => handleDeleteTask(task.id)}
              >
                Delete
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-message">No tasks to display</p>
      )}
      
      {/* Summary footer */}
      <div className="task-summary">
        <p>
          {tasks.filter(task => !task.completed).length} tasks remaining
        </p>
      </div>
    </>
  );
};

// PropTypes for component props validation
TaskList.propTypes = {
  initialTasks: PropTypes.arrayOf(
    PropTypes.shape({
      id: PropTypes.number.isRequired,
      text: PropTypes.string.isRequired,
      completed: PropTypes.bool.isRequired
    })
  )
};

// Class-based component example with lifecycle methods
class TaskApp extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      savedTasks: []
    };
  }

  componentDidMount() {
    // Simulate loading saved tasks from localStorage
    const savedTasks = JSON.parse(localStorage.getItem('tasks') || '[]');
    this.setState({ savedTasks });
  }

  handleTasksUpdate = (updatedTasks) => {
    // Save to localStorage and update state
    localStorage.setItem('tasks', JSON.stringify(updatedTasks));
    this.setState({ savedTasks: updatedTasks });
  };

  render() {
    return (
      <div className="task-app">
        <TaskList 
          initialTasks={this.state.savedTasks} 
          onTasksUpdate={this.handleTasksUpdate}
        />
      </div>
    );
  }
}

export default TaskApp;

