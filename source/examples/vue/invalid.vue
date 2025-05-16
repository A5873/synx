<!-- 
  Invalid Vue.js Single-File Component
  This file contains intentional errors to test Synx validation
-->
<template>
  <!-- Missing closing div tag -->
  <div class="app">
    <!-- Invalid v-for syntax (missing :key) -->
    <ul>
      <li v-for="item in items">
        {{ item.name }}
      </li>
    </ul>
    
    <!-- Mismatched closing tag -->
    <h1>{{ title }}<h2>
    
    <!-- Invalid event binding (missing method name) -->
    <button @click=>Click me</button>
    
    <!-- Invalid v-model usage (missing expression) -->
    <input v-model>
    
    <!-- Unclosed attribute bracket -->
    <div :class="{ active: isActive" ></div>
    
    <!-- Missing quotation marks in v-bind -->
    <img :src=imageSrc>
    
    <!-- Invalid template directive -->
    <div v-incorrect="value"></div>
    
    <!-- Invalid mustache template syntax -->
    <p>{{ message }</p>
    
    <!-- Incorrect slot syntax -->
    <template slot>
      Slot content
    </template>
    
    <!-- Invalid directive argument -->
    <div v-bind:class.="active"></div>
    
    <!-- Missing v-if expression -->
    <div v-if></div>
    
    <!-- Invalid v-on modifier -->
    <button @click.invalid="handleClick"></button>
    
    <!-- Invalid directive combination -->
    <input v-if="true" v-else>
    
    <!-- Mixing v-for with same element v-if without using key -->
    <div v-for="user in users" v-if="user.isActive">
      {{ user.name }}
    </div>
  </div>
</template>

<script>
// Missing export default
{
  // Missing comma after property
  name: 'InvalidComponent'
  
  // Incorrect data property (not a function in component)
  data: {
    title: 'Invalid Vue Component',
    items: [
      { id: 1, name: 'Item 1' },
      { id: 2, name: 'Item 2' },
      { id: 3, name: 'Item 3' },
    ],
    imageSrc: 'https://example.com/image.jpg',
    message: 'Hello world',
    isActive: true
  },
  
  // Methods without comma separation
  methods: {
    // Missing function body
    handleClick()
    
    // Missing comma after method
    logMessage() {
      console.log(this.message)
    }
    
    increment: function() {
      counter++  // undefined variable
    }
  }
  
  // Invalid lifecycle hook (typo)
  mouted() {
    console.log('Component mounted')
  },
  
  // Invalid computed property (not a function)
  computed: {
    fullMessage: 'Computed: ' + this.message  // not allowed to use this directly
  },
  
  // Misspelled property
  compnents: {
    'app-child': {
      template: '<div>Child Component</div>'
    }
  },
  
  // Incorrect prop validation
  props: {
    propA: Number
    propB: {
      // Missing required property type
      required: true
    }
  }
}
</script>

<style>
/* Missing closing bracket */
.app {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;

/* Unclosed rule block */
.active {
  color: blue;
  font-weight: bold;

/* Invalid CSS selector */
.item:hover::: {
  background-color: #eee;
}

/* Missing semicolon */
.button {
  padding: 10px
  border: none;
}

/* Extra closing bracket */
ul {
  list-style: none;
  margin: 0;
  padding: 0;
}}

/* Improper nesting (CSS, not SCSS) */
.parent {
  color: red;
  .child {
    color: blue;
  }
}

/* Invalid property value */
.error {
  color: error;
  font-size: 12px;;
}

/* Missing closing brace for style tag */
</style>

