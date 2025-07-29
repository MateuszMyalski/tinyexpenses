
/**
 * @file csv_editor.js
 * @copyright Copyright (c) 2024 mmyalski. All rights reserved.
 * @description Provides interactive CSV table editing functionality, including row insertion, deletion, cell modification tracking, and data serialization for form submission.
 */

const TABLE_ID = "csv-edit-table";
const DATA_CELL_CLASS = "data";  // class for serializable <td>
const EDITABLE_CELL_RANGE = [3, 6];  // inclusive: category to description
const TIMESTAMP_INDEX = 2;
const ROW_INDEX = 1;
const SOURCE_ROW_CLASS = "source";

const diff = {
    addedRows: 0,
    removedRows: 0,
    modifiedCells: 0,
    prevTableData: ""
};

function deleteRow(button) {
    diff.modifiedCells += getCountOfModifiedCells();

    const row = button.closest("tr");
    row.remove();
    updateRowNumbers();

    diff.prevTableData = getTableData();

    if (row.classList.contains(SOURCE_ROW_CLASS)) {
        diff.removedRows += 1;
    } else {
        diff.addedRows -= 1;
    }
}

function insertRow(button) {
    diff.modifiedCells += getCountOfModifiedCells();

    const currentRow = button.closest("tr");
    const newRow = currentRow.cloneNode(true);
    newRow.className = "";

    const cells = newRow.querySelectorAll("td");
    for (let i = EDITABLE_CELL_RANGE[0]; i <= EDITABLE_CELL_RANGE[1]; i++) {
        cells[i].innerText = "";
    }
    cells[TIMESTAMP_INDEX].innerText = getCurrentTimestamp();

    currentRow.parentNode.insertBefore(newRow, currentRow.nextSibling);
    updateRowNumbers();

    diff.prevTableData = getTableData();
    diff.addedRows += 1;
}

function updateRowNumbers() {
    const rows = document.querySelectorAll(`#${TABLE_ID} tbody tr`);
    rows.forEach((row, index) => {
        row.cells[ROW_INDEX].innerText = index + 1;
    });
}

function getTableData() {
    const rows = document.querySelectorAll(`#${TABLE_ID} tbody tr`);
    return Array.from(rows).map(row =>
        Array.from(row.querySelectorAll(`td.${DATA_CELL_CLASS}`)).map(td => td.textContent.trim())
    );
}

function getCountOfModifiedCells() {
    const current = getTableData();
    const original = diff.prevTableData;
    let modified = 0;

    const minLen = Math.min(original.length, current.length);
    for (let i = 0; i < minLen; i++) {
        const origRow = original[i];
        const currRow = current[i];
        for (let j = 0; j < origRow.length; j++) {
            if ((origRow[j] || '') !== (currRow[j] || '')) {
                modified++;
            }
        }
    }

    return modified;
}

function getCurrentTimestamp() {
    return new Date().toISOString().slice(0, 19).replace("T", " ");
}

window.addEventListener("DOMContentLoaded", () => {
    diff.prevTableData = getTableData();

    const form = document.querySelector("form");
    if (!form) return;

    form.addEventListener("submit", function (event) {
        diff.modifiedCells += getCountOfModifiedCells();

        const msg = `Edited cells: ${diff.modifiedCells}\nNew rows: ${diff.addedRows}\nRemoved rows: ${diff.removedRows}\n\nProceed with saving changes?`;
        if (!confirm(msg)) {
            event.preventDefault();
            return;
        }

        const rows = document.querySelectorAll(`#${TABLE_ID} tbody tr`);
        const data = Array.from(rows).map(row =>
            Array.from(row.querySelectorAll(`td.${DATA_CELL_CLASS}`)).map(td => td.textContent.trim())
        );

        const hiddenInput = form.querySelector("[name=table_data]");
        if (hiddenInput) {
            hiddenInput.value = JSON.stringify(data);
        }
    });
});
