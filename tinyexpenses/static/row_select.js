
/**
 * @file csv_editor.js
 * @copyright Copyright (c) 2024 mmyalski. All rights reserved.
 * @description Allows to select row in long tables.
 */

document.querySelectorAll('tr.category-row').forEach(row => {
    row.addEventListener('click', (e) => {
        document.querySelectorAll('tr.category-row').forEach(r => r.classList.remove('active-hover'));

        row.classList.add('active-hover');
        e.stopPropagation();
    });
});;

document.body.addEventListener('click', () => {
    document.querySelectorAll('tr.category-row').forEach(r => r.classList.remove('active-hover'));
});