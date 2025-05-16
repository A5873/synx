// Main JavaScript file for the Synx Demo App

// Wait for the DOM to be fully loaded
document.addEventListener('DOMContentLoaded', function() {
    console.log('Synx Demo App initialized');
    
    // Initialize app components
    initializeApp();
    
    // Add event listeners
    setupEventListeners();
});

/**
 * Initialize the application components
 */
function initializeApp() {
    // Set up the current theme based on user preference
    const prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (prefersDarkMode) {
        document.body.classList.add('dark-theme');
    }
    
    // Check if user has visited before
    const hasVisitedBefore = localStorage.getItem('hasVisited');
    if (!hasVisitedBefore) {
        showWelcomeMessage();
        localStorage.setItem('hasVisited', 'true');
    }
}

/**
 * Set up all event listeners for the app
 */
function setupEventListeners() {
    // Load data button
    const loadDataBtn = document.getElementById('loadDataBtn');
    if (loadDataBtn) {
        loadDataBtn.addEventListener('click', function() {
            loadData();
        });
    }
    
    // Mobile menu toggle (for responsive design)
    const menuToggle = document.querySelector('.menu-toggle');
    if (menuToggle) {
        menuToggle.addEventListener('click', function() {
            const navLinks = document.querySelector('.nav-links');
            navLinks.classList.toggle('active');
        });
    }
    
    // Add scroll effects
    window.addEventListener('scroll', function() {
        const header = document.querySelector('.header');
        if (window.scrollY > 50) {
            header.classList.add('scrolled');
        } else {
            header.classList.remove('scrolled');
        }
    });
}

/**
 * Shows a welcome message for first-time visitors
 */
function showWelcomeMessage() {
    const welcomeMessage = document.createElement('div');
    welcomeMessage.className = 'welcome-message';
    welcomeMessage.innerHTML = `
        <h3>Welcome to the Synx Demo!</h3>
        <p>This is your first visit. Explore our features to see how Synx can validate different file types.</p>
        <button class="btn close-btn">Got it</button>
    `;
    
    document.body.appendChild(welcomeMessage);
    
    // Add event listener to close button
    const closeBtn = welcomeMessage.querySelector('.close-btn');
    closeBtn.addEventListener('click', function() {
        welcomeMessage.remove();
    });
    
    // Auto-hide after 5 seconds
    setTimeout(function() {
        welcomeMessage.classList.add('hiding');
        setTimeout(function() {
            welcomeMessage.remove();
        }, 500);
    }, 5000);
}

/**
 * Handles errors with a user-friendly message
 * @param {Error} error - The error object
 */
function handleError(error) {
    console.error('An error occurred:', error);
    
    const errorContainer = document.createElement('div');
    errorContainer.className = 'error-notification';
    errorContainer.textContent = 'Something went wrong. Please try again later.';
    
    document.body.appendChild(errorContainer);
    
    // Auto-remove after 3 seconds
    setTimeout(function() {
        errorContainer.remove();
    }, 3000);
}

