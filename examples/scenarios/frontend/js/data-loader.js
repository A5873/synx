// Data loader module for the Synx Demo App

/**
 * Loads sample data from a JSON configuration file
 */
function loadData() {
    console.log('Loading data...');
    
    // Show loading indicator
    const dataContainer = document.getElementById('dataContainer');
    if (dataContainer) {
        dataContainer.innerHTML = '<div class="loading">Loading data...</div>';
    }
    
    // Simulate API call delay
    setTimeout(() => {
        fetch('config/data.json')
            .then(response => {
                if (!response.ok) {
                    throw new Error('Network response was not ok');
                }
                return response.json();
            })
            .then(data => {
                displayData(data);
            })
            .catch(error => {
                handleError(error);
            });
    }, 1000);
}

/**
 * Display the loaded data in the container
 * @param {Object} data - The JSON data to display
 */
function displayData(data) {
    const dataContainer = document.getElementById('dataContainer');
    if (!dataContainer) return;
    
    // Clear loading indicator
    dataContainer.innerHTML = '';
    
    // Create data display
    if (data && data.items && Array.isArray(data.items)) {
        // Create header
        const header = document.createElement('h3');
        header.textContent = data.title || 'Loaded Data';
        dataContainer.appendChild(header);
        
        // Create list of items
        const list = document.createElement('ul');
        list.className = 'data-list';
        
        data.items.forEach(item => {
            const listItem = document.createElement('li');
            listItem.innerHTML = `
                <strong>${item.name}</strong>
                <p>${item.description}</p>
            `;
            list.appendChild(listItem);
        });
        
        dataContainer.appendChild(list);
    } else {
        dataContainer.innerHTML = '<p>No data available or invalid format</p>';
    }
}

/**
 * Handle errors during data loading
 * @param {Error} error - The error that occurred
 */
function handleError(error) {
    console.error('Error loading data:', error);
    
    const dataContainer = document.getElementById('dataContainer');
    if (dataContainer) {
        dataContainer.innerHTML = `
            <div class="error">
                <h3>Error Loading Data</h3>
                <p>${error.message || 'An unknown error occurred'}</p>
                <button id="retryBtn" class="btn primary">Retry</button>
            </div>
        `;
        
        // Add retry functionality
        const retryBtn = document.getElementById('retryBtn');
        if (retryBtn) {
            retryBtn.addEventListener('click', loadData);
        }
    }
}

