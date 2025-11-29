// Sample JavaScript file for testing call graph extraction

function processData(data) {
    validateInput(data);
    const result = transformData(data);
    saveToDatabase(result);
    return result;
}

function validateInput(input) {
    if (!input) {
        throw new Error('Invalid input');
    }
    return true;
}

function transformData(data) {
    return data.map(item => item * 2);
}

function saveToDatabase(data) {
    console.log('Saving:', data);
}

const handleClick = () => {
    const data = fetchData();
    processData(data);
};

const fetchData = () => {
    return [1, 2, 3];
};
