/**
 * A valid TypeScript React (TSX) file showcasing typed components
 * This file should pass validation with Synx
 */
import React, { useState, useEffect, useReducer, useCallback } from 'react';

// TypeScript interfaces for type checking
interface Todo {
  id: number;
  text: string;
  completed: boolean;
  category?: string;
  priority: 'low' | 'medium' | 'high';
  tags: string[];
}

interface TodosState {
  todos: Todo[];
  filter: 'all' | 'active' | 'completed';
  loading: boolean;
  error: string | null;
}

// Discriminated union type for actions
type TodoAction =
  | { type: 'ADD_TODO'; payload: Omit<Todo, 'id'> }
  | { type: 'TOGGLE_TODO'; payload: number }
  | { type: 'DELETE_TODO'; payload: number }
  | { type: 'SET_FILTER'; payload: TodosState['filter'] }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null };

// Props interface with generics
interface TodoListProps<T> {
  initialTodos?: Todo[];
  title: string;
  onTodoAdded?: (todo: Todo) => void;
  renderItem?: (item: T) => React.ReactNode;
  children?: React.ReactNode;
}

// Component props with default values and function types
interface TodoItemProps {
  todo: Todo;
  onToggle: (id: number) => void;
  onDelete: (id: number) => void;
  onEdit?: (id: number, newText: string) => void;
}

// Reducer function for state management
const todoReducer = (state: TodosState, action: TodoAction): TodosState => {
  switch (action.type) {
    case 'ADD_TODO':
      return {
        ...state,
        todos: [
          ...state.todos,
          {
            id: Date.now(),
            ...action.payload
          }
        ]
      };
    case 'TOGGLE_TODO':
      return {
        ...state,
        todos: state.todos.map(todo =>
          todo.id === action.payload ? { ...todo, completed: !todo.completed } : todo
        )
      };
    case 'DELETE_TODO':
      return {
        ...state,
        todos: state.todos.filter(todo => todo.id !== action.payload)
      };
    case 'SET_FILTER':
      return {
        ...state,
        filter: action.payload
      };
    case 'SET_LOADING':
      return {
        ...state,
        loading: action.payload
      };
    case 'SET_ERROR':
      return {
        ...state,
        error: action.payload
      };
    default:
      return state;
  }
};

// Generic type constraint example
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

// TodoItem component with type annotations
const TodoItem: React.FC<TodoItemProps> = ({ todo, onToggle, onDelete, onEdit }) => {
  const [isEditing, setIsEditing] = useState<boolean>(false);
  const [editText, setEditText] = useState<string>(todo.text);
  
  const handleEdit = (): void => {
    if (onEdit && editText.trim() !== '') {
      onEdit(todo.id, editText);
      setIsEditing(false);
    }
  };
  
  return (
    <li className={`todo-item ${todo.completed ? 'completed' : ''}`}>
      <input
        type="checkbox"
        checked={todo.completed}
        onChange={() => onToggle(todo.id)}
      />
      
      {isEditing ? (
        <>
          <input
            type="text"
            value={editText}
            onChange={(e) => setEditText(e.target.value)}
            onBlur={handleEdit}
            autoFocus
          />
          <button onClick={handleEdit}>Save</button>
        </>
      ) : (
        <>
          <span 
            className="todo-text"
            style={{ textDecoration: todo.completed ? 'line-through' : 'none' }}
          >
            {todo.text}
          </span>
          <span className={`priority ${todo.priority}`}>{todo.priority}</span>
          <button onClick={() => setIsEditing(true)}>Edit</button>
        </>
      )}
      
      <button onClick={() => onDelete(todo.id)}>Delete</button>
      
      {todo.tags && todo.tags.length > 0 && (
        <div className="tags">
          {todo.tags.map((tag, index) => (
            <span key={index} className="tag">{tag}</span>
          ))}
        </div>
      )}
    </li>
  );
};

// Main TodoList component with generics, hooks, and TypeScript features
function TodoList<T extends Todo>({
  initialTodos = [],
  title,
  onTodoAdded,
  renderItem,
  children
}: TodoListProps<T>): JSX.Element {
  // useReducer with TypeScript
  const [state, dispatch] = useReducer(todoReducer, {
    todos: initialTodos,
    filter: 'all',
    loading: false,
    error: null
  });
  
  const [newTodoText, setNewTodoText] = useState<string>('');
  const [selectedPriority, setSelectedPriority] = useState<Todo['priority']>('medium');
  
  // useEffect with proper typing
  useEffect(() => {
    const fetchData = async (): Promise<void> => {
      try {
        dispatch({ type: 'SET_LOADING', payload: true });
        
        // Simulated API call
        const response = await new Promise<Todo[]>(resolve => {
          setTimeout(() => resolve(initialTodos), 1000);
        });
        
        dispatch({ type: 'SET_LOADING', payload: false });
      } catch (error) {
        dispatch({ type: 'SET_ERROR', payload: 'Failed to fetch todos' });
        dispatch({ type: 'SET_LOADING', payload: false });
      }
    };
    
    fetchData();
    
    return () => {
      // Cleanup
      console.log('Component unmounted');
    };
  }, [initialTodos]);
  
  // Type safe callbacks
  const handleAddTodo = useCallback((): void => {
    if (newTodoText.trim() === '') return;
    
    const newTodo: Omit<Todo, 'id'> = {
      text: newTodoText,
      completed: false,
      priority: selectedPriority,
      tags: newTodoText.match(/#[a-zA-Z0-9]+/g)?.map(tag => tag.slice(1)) || []
    };
    
    dispatch({ type: 'ADD_TODO', payload: newTodo });
    
    if (onTodoAdded) {
      onTodoAdded({ ...newTodo, id: Date.now() });
    }
    
    setNewTodoText('');
  }, [newTodoText, selectedPriority, onTodoAdded]);
  
  // Computed values with filtered results
  const filteredTodos = state.todos.filter(

