// Generate a large, valid JavaScript translation file for performance testing

const fs = require('fs');

// Base categories for realistic translation keys
const categories = [
  'auth', 'user', 'dashboard', 'settings', 'profile', 'notifications', 
  'messages', 'calendar', 'tasks', 'projects', 'reports', 'analytics',
  'billing', 'support', 'help', 'admin', 'security', 'privacy',
  'forms', 'validation', 'buttons', 'navigation', 'search', 'filters'
];

const subcategories = [
  'labels', 'placeholders', 'tooltips', 'errors', 'success', 'warnings',
  'confirmations', 'titles', 'descriptions', 'instructions', 'hints'
];

const actions = [
  'create', 'edit', 'delete', 'save', 'cancel', 'submit', 'reset',
  'view', 'download', 'upload', 'share', 'export', 'import',
  'approve', 'reject', 'archive', 'restore', 'duplicate', 'move'
];

// Sample text variations
const texts = [
  'Click here to continue',
  'Please enter a valid email address',
  'Your changes have been saved successfully',
  'Are you sure you want to delete this item?',
  'This field is required',
  'Loading data, please wait...',
  'No results found for your search',
  'You do not have permission to perform this action',
  'The operation completed successfully',
  'An unexpected error occurred',
  'Please try again later',
  'Your session has expired',
  'File uploaded successfully',
  'Invalid file format',
  'Maximum file size exceeded',
  'Connection timeout',
  'Server is temporarily unavailable',
  'Data has been updated',
  'Changes saved automatically',
  'Please confirm your password'
];

let translations = {};

// Generate nested structure
categories.forEach((category, catIndex) => {
  translations[category] = {};
  
  subcategories.forEach((subcat, subIndex) => {
    translations[category][subcat] = {};
    
    actions.forEach((action, actionIndex) => {
      // Create unique keys and varied text content
      const textIndex = (catIndex + subIndex + actionIndex) % texts.length;
      const baseText = texts[textIndex];
      
      // Add some variation to make it more realistic
      const variations = [
        baseText,
        baseText + ' for ' + category,
        baseText.replace('item', category),
        baseText + ' in ' + subcat + ' section',
        `${action.charAt(0).toUpperCase() + action.slice(1)}: ${baseText}`
      ];
      
      const finalText = variations[(catIndex * subIndex * actionIndex) % variations.length];
      
      translations[category][subcat][action] = finalText;
      
      // Add some additional nested levels for complexity
      if (actionIndex % 3 === 0) {
        translations[category][subcat][action + '_details'] = {
          short: finalText.substring(0, 20) + '...',
          long: finalText + ' This is additional detailed information about the ' + action + ' operation.',
          help: 'Need help with ' + action + '? Contact support for assistance.'
        };
      }
    });
  });
  
  // Add some arrays for variety
  translations[category]['options'] = [
    'Option A for ' + category,
    'Option B for ' + category, 
    'Option C for ' + category,
    'Custom option for ' + category
  ];
  
  // Add some multi-line strings
  translations[category]['multiline'] = {
    description: `This is a comprehensive description for ${category} 
      that spans multiple lines to provide detailed information
      about the functionality and usage guidelines`,
    instructions: 'Step 1: Navigate to ' + category + ' section\\n' +
      'Step 2: Select the appropriate option\\n' +
      'Step 3: Confirm your selection'
  };
});

// Generate the JavaScript file content
const jsContent = `export default ${JSON.stringify(translations, null, 2)};`;

// Write to file
fs.writeFileSync('tests/fixtures/performance/large-en.js', jsContent);

console.log('Generated large translation file with:');
console.log('- Categories:', categories.length);
console.log('- Subcategories per category:', subcategories.length);
console.log('- Actions per subcategory:', actions.length);
console.log('- Total approximate keys:', categories.length * subcategories.length * actions.length * 1.5);
console.log('- File size:', Math.round(jsContent.length / 1024) + ' KB');