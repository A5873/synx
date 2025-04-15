<template>
  <!-- 
    A valid Vue.js Single-File Component
    This file should pass validation with Synx
  -->
  <div id="app" class="todo-app">
    <header class="app-header">
      <h1>{{ title }}</h1>
      <p>A simple Todo application built with Vue.js</p>
    </header>

    <!-- Component with slots and props -->
    <section-container title="Add New Task">
      <form @submit.prevent="addTodo" class="task-form">
        <div class="form-group">
          <label for="task-input">Task Name</label>
          <input 
            id="task-input"
            v-model="newTodo"
            type="text"
            placeholder="What needs to be done?"
            required
          />
        </div>
        
        <div class="form-group">
          <label for="priority-select">Priority</label>
          <select id="priority-select" v-model="newTodoPriority">
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>
        
        <div class="form-group">
          <label for="due-date">Due Date</label>
          <input 
            id="due-date"
            v-model="dueDate"
            type="date"
          />
        </div>
        
        <button type="submit" :disabled="!newTodo.trim()">
          Add Task
        </button>
      </form>
    </section-container>

    <!-- Filter controls with event binding -->
    <div class="filters">
      <button 
        v-for="filter in filters" 
        :key="filter.value"
        @click="currentFilter = filter.value"
        :class="{ active: currentFilter === filter.value }"
      >
        {{ filter.label }}
      </button>
    </div>

    <!-- Component with v-if, v-for, and dynamic attributes -->
    <section-container title="Tasks">
      <p v-if="filteredTodos.length === 0" class="empty-list">
        No tasks to display
      </p>
      
      <transition-group name="task-list" tag="ul" class="task-list">
        <li v-for="todo in filteredTodos" :key="todo.id" class="task-item">
          <div class="task-content">
            <input 
              type="checkbox" 
              :id="'todo-' + todo.id"
              v-model="todo.completed"
              @change="saveTodos"
            />
            <label 
              :for="'todo-' + todo.id"
              :class="{ 'completed': todo.completed }"
            >
              {{ todo.text }}
            </label>
            
            <!-- Dynamic CSS classes -->
            <span 
              class="priority-badge"
              :class="'priority-' + todo.priority"
            >
              {{ todo.priority }}
            </span>
            
            <!-- Conditional rendering -->
            <span v-if="todo.dueDate" class="due-date">
              Due: {{ formatDate(todo.dueDate) }}
            </span>
          </div>
          
          <div class="task-actions">
            <!-- Custom event handler -->
            <button @click="editTodo(todo)" class="edit-btn">
              Edit
            </button>
            <button @click="removeTodo(todo)" class="delete-btn">
              Delete
            </button>
          </div>
        </li>
      </transition-group>
      
      <!-- Slot for custom content -->
      <template #footer>
        <div class="task-summary">
          <p>{{ activeTodosCount }} items left</p>
          <button 
            v-if="completedTodosCount > 0"
            @click="clearCompleted"
            class="clear-btn"
          >
            Clear completed
          </button>
        </div>
      </template>
    </section-container>
    
    <!-- Modal component with teleport (Vue 3 feature) -->
    <teleport to="body" v-if="isEditModalOpen">
      <div class="modal-overlay">
        <div class="modal">
          <h2>Edit Task</h2>
          <form @submit.prevent="updateTodo">
            <div class="form-group">
              <label for="edit-task-input">Task</label>
              <input 
                id="edit-task-input"
                v-model="editingTodo.text"
                type="text"
                required
              />
            </div>
            
            <div class="form-group">
              <label for="edit-priority-select">Priority</label>
              <select id="edit-priority-select" v-model="editingTodo.priority">
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
              </select>
            </div>
            
            <div class="modal-actions">
              <button type="button" @click="closeEditModal">Cancel</button>
              <button type="submit">Save</button>
            </div>
          </form>
        </div>
      </div>
    </teleport>
  </div>
</template>

<script>
// Options API example
export default {
  name: 'TodoApp',
  
  // Component registration
  components: {
    SectionContainer: {
      props: {
        title: {
          type: String,
          required: true
        }
      },
      template: `
        <section class="section-container">
          <h2>{{ title }}</h2>
          <div class="section-content">
            <slot></slot>
          </div>
          <div class="section-footer">
            <slot name="footer"></slot>
          </div>
        </section>
      `
    }
  },
  
  // Props definition
  props: {
    storageKey: {
      type: String,
      default: 'vue-todos'
    }
  },
  
  // Data properties
  data() {
    return {
      title: 'Vue Todo App',
      todos: [],
      newTodo: '',
      newTodoPriority: 'medium',
      dueDate: '',
      currentFilter: 'all',
      filters: [
        { label: 'All', value: 'all' },
        { label: 'Active', value: 'active' },
        { label: 'Completed', value: 'completed' }
      ],
      isEditModalOpen: false,
      editingTodo: null,
    };
  },
  
  // Computed properties
  computed: {
    // Filter todos based on current filter
    filteredTodos() {
      if (this.currentFilter === 'active') {
        return this.todos.filter(todo => !todo.completed);
      }
      if (this.currentFilter === 'completed') {
        return this.todos.filter(todo => todo.completed);
      }
      return this.todos;
    },
    
    // Count active todos
    activeTodosCount() {
      return this.todos.filter(todo => !todo.completed).length;
    },
    
    // Count completed todos
    completedTodosCount() {
      return this.todos.filter(todo => todo.completed).length;
    }
  },
  
  // Watchers
  watch: {
    // Save todos when they change
    todos: {
      handler() {
        this.saveTodos();
      },
      deep: true
    }
  },
  
  // Lifecycle hooks
  created() {
    // Load todos from localStorage
    this.loadTodos();
  },
  
  mounted() {
    // Focus input when component is mounted
    document.getElementById('task-input')?.focus();
  },
  
  // Methods
  methods: {
    // Add new todo
    addTodo() {
      if (!this.newTodo.trim()) return;
      
      this.todos.push({
        id: Date.now(),
        text: this.newTodo.trim(),
        completed: false,
        priority: this.newTodoPriority,
        dueDate: this.dueDate || null,
        createdAt: new Date()
      });
      
      this.newTodo = '';
      this.newTodoPriority = 'medium';
      this.dueDate = '';
      this.saveTodos();
    },
    
    // Remove todo
    removeTodo(todo) {
      const index = this.todos.findIndex(item => item.id === todo.id);
      if (index !== -1) {
        this.todos.splice(index, 1);
        this.saveTodos();
      }
    },
    
    // Clear completed todos
    clearCompleted() {
      this.todos = this.todos.filter(todo => !todo.completed);
      this.saveTodos();
    },
    
    // Edit todo
    editTodo(todo) {
      this.editingTodo = { ...todo };
      this.isEditModalOpen = true;
    },
    
    // Update todo
    updateTodo() {
      if (!this.editingTodo.text.trim()) return;
      
      const index = this.todos.findIndex(todo => todo.id === this.editingTodo.id);
      if (index !== -1) {
        this.todos[index] = { ...this.editingTodo };
        this.closeEditModal();
        this.saveTodos();
      }
    },
    
    // Close edit modal
    closeEditModal() {
      this.isEditModalOpen = false;
      this.editingTodo = null;
    },
    
    // Save todos to localStorage
    saveTodos() {
      localStorage.setItem(this.storageKey, JSON.stringify(this.todos));
    },
    
    // Load todos from localStorage
    loadTodos() {
      try {
        const savedTodos = localStorage.getItem(this.storageKey);
        if (savedTodos) {
          this.todos = JSON.parse(savedTodos);
        }
      } catch (e) {
        console.error('Failed to load todos from localStorage:', e);
        this.todos = [];
      }
    },
    
    // Format date
    formatDate(dateString) {
      if (!dateString) return '';
      const date = new Date(dateString);
      return date.toLocaleDateString();
    }
  }
};
</script>

<style>
/* Global styles */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: 'Avenir', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  background-color: #f5f7fa;
  line-height: 1.6;
}

.todo-app {
  max-width: 600px;
  margin: 0 auto;
  padding: 2rem;
}

.app-header {
  text-align: center;
  margin-bottom: 2rem;
}

.app-header h1 {
  color: #41b883;
  font-size: 2.5rem;
  margin-bottom: 0.5rem;
}

.section-container {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
  overflow: hidden;
}

.section-container h2 {
  background-color: #41b883;
  color: white;
  padding: 1rem;
  font-size: 1.25rem;
}

.section-content {
  padding: 1.5rem;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: bold;
}

input, select, button {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
}

button {
  background-color: #41b883;
  color: white;
  border: none;
  cursor: pointer;
  transition: background-color 0.3s;
}

button:hover {
  background-color: #359268;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.filters {
  display: flex;
  justify-content: center;
  margin-bottom: 1rem;
}

.filters button {
  background-color: transparent;
  color: #2c3e50;
  border: 1px solid #ddd;
  margin: 0 0.5rem;
  padding: 0.5rem 1rem;
  width: auto;
}

.filters button.active {
  background-color: #41b883;
  color: white;
  border-color: #41b883;
}

.task-list {
  list-style: none;
}

.task-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 0;
  border-bottom: 1px solid #eee;
}

.task-content {
  display: flex;
  align-items: center;
  flex: 1;
}

.task-content label {
  margin-left: 0.5rem;
  flex: 1;
}

.task-content label.completed {
  text-decoration: line-through;
  color: #999;
}

.priority-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  margin-left: 0.5rem;
}

.priority-low {
  background-color: #4caf50;
  color: white;
}

.priority-medium {
  background-color: #ff9800;
  color: white;
}

.priority-high {
  background-color: #f44336;
  color: white;
}

.due-date {
  font-size: 0.75rem;
  color: #666;
  margin-left: 0.5rem;
}

.task-actions {
  display: flex;
}

.task-actions button {
  width: auto;
  padding: 0.25rem 0.5rem;
  margin-left: 0.5rem;
  font-size: 0.875rem;
}

.edit-btn {
  background-color: #2196f3;
}

.delete-btn {
  background-color: #f44336;
}

.task-summary {
  display: flex;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  border-top: 1px solid #eee;
}

.clear-btn {
  background-color: transparent;
  color: #f44336;
  border: 1px solid #f44336;
  padding: 0.25rem 0.5rem;
  width: auto;
  font-size: 0.875rem;
}

.clear-btn:hover {
  background-color: #f44336;
  color: white;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal {
  background-color: white;
  border-radius: 8px;
  padding: 1.5rem;
  width: 90%;
  max-width: 500px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
}

.modal h2 {
  margin-bottom: 1.5rem;
  color: #41b883;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 1.5rem;
}

.modal-actions button {
  width: auto;
  padding: 0.5rem 1rem;
  margin-left: 0.5rem;
}

.modal-actions button[type="button"] {
  background-color: #ccc;
}

/* Transition effects */
.task-list-enter-active,
.task-list-leave-active {
  transition: all 0.3s;
}

.task-list-enter-from,
.task-list-leave-to {
  opacity: 0;
  transform: translateY(30px);
}

/* Empty list message */
.empty-list {
  text-align: center;
  color: #999;
  padding: 2rem 0;
}
</style>
