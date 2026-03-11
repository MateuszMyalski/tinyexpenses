/**
 * @file csv_editor.js
 * @copyright Copyright (c) 2026 mmyalski. All rights reserved.
 * @description Use tables to edit CSV files and send them back to the server.
 *              Tables are auto-initialized via [data-csv-table] attributes.
 *
 * Usage:
 *   <table id="my-table" data-csv-table data-form="my-form" data-container="[name=csv_content]">
 *   <div data-csv-controls="my-table">
 *     <button data-action="add-row">+ Add row</button>
 *     <button data-action="submit">Submit</button>
 *   </div>
 *   <form id="my-form" method="POST" hidden>
 *     <input type="hidden" name="csv_content" />
 *   </form>
 */

class CsvTable {
    static #DELETE_BUTTON_HTML = `<button data-tooltip="Remove" type="submit">🗑️</button>`;

    constructor(tableId) {
        this.table = document.getElementById(tableId);
        if (!this.table) throw new Error(`CsvTable: table "#${tableId}" not found.`);

        this.formId = this.table.dataset.form;
        this.containerSelector = this.table.dataset.container;
    }

    // --- Private helpers ---

    #rows() {
        return this.table.querySelectorAll("tbody tr");
    }

    #tbody() {
        return this.table.querySelector("tbody");
    }

    #dataHeaders() {
        return this.table.querySelectorAll("thead th.data-col");
    }

    #createDeleteButton(row) {
        const cell = row.insertCell(0);
        cell.innerHTML = CsvTable.#DELETE_BUTTON_HTML;
        cell.querySelector("button").addEventListener("click", () => {
            if (confirm("Do you want to delete the row?"))
                row.remove()
        }
        );
    }

    // --- Public API ---

    getTableContent() {
        return Array.from(this.#rows()).map(row =>
            row.querySelectorAll("td.data")
        );
    }

    serialize() {
        return JSON.stringify(
            this.getTableContent().map(row => {
                const result = [];
                const groups = new Map();

                Array.from(row).forEach(td => {
                    const marker = td.dataset.marker;
                    const value = td.textContent.trim();

                    console.log("Pus" + marker)
                    if (!marker) {
                        result.push(value);
                    } else {
                        if (!groups.has(marker)) {
                            groups.set(marker, []);
                            result.push(groups.get(marker));
                        }
                        groups.get(marker).push(value);
                    }
                });
                console.log(result)
                return result;
            })
        );
    }

    send() {
        const form = document.getElementById(this.formId);
        if (!form) {
            alert(`Form "#${this.formId}" not found!`);
            return;
        }

        const container = form.querySelector(this.containerSelector);
        if (!container) {
            alert(`Container "${this.containerSelector}" not found in form "#${this.formId}"!`);
            return;
        }

        container.value = this.serialize();

        if (confirm("Do you want to overwrite the file?")) {
            form.submit();
        }
    }

    addDeleteButtons() {
        this.#rows().forEach(row => this.#createDeleteButton(row));
    }

    appendRow() {
        const row = this.#tbody().insertRow();

        this.#dataHeaders().forEach(th => {
            const cell = row.insertCell();

            var defaultValue = th.dataset.default;
            if (defaultValue == null) {
                defaultValue = `New ${th.textContent}`;
            }

            if (th.dataset.marker) {
                cell.dataset.marker = th.dataset.marker;
            }

            cell.textContent = defaultValue;

            cell.classList.add("data");

            if (th.classList.contains("editable")) {
                cell.contentEditable = "true";
            }
        });

        this.#createDeleteButton(row);
    }

    // --- Static auto-initialization ---

    static init() {
        document.querySelectorAll("[data-csv-table]").forEach(table => {
            let editor;

            try {
                editor = new CsvTable(table.id);
            } catch (e) {
                console.error(e);
                return;
            }

            editor.addDeleteButtons();

            const controls = document.querySelector(`[data-csv-controls="${table.id}"]`);
            if (!controls) return;

            controls.querySelector("[data-action=add-row]")
                ?.addEventListener("click", () => editor.appendRow());

            controls.querySelector("[data-action=submit]")
                ?.addEventListener("click", () => editor.send());
        });
    }
}