
/**
 * @file csv_editor.js
 * @copyright Copyright (c) 2024 mmyalski. All rights reserved.
 * @description Provides minimalistic bar graph displayed inside table row for Needs/Wants/Savings balances
 */

document.addEventListener("DOMContentLoaded", function () {
    const barColors = {};
    Object.keys(year_expenses).forEach(type => {
        barColors[type] = getComputedStyle(document.querySelector("tr.category-row." + type)).backgroundColor;
    });

    const canvas = document.getElementById("regionBar");
    const ctx = canvas.getContext("2d");

    // Ensure canvas scales with device pixel ratio
    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;
    canvas.width = width;
    canvas.height = height;

    const total = Object.values(year_expenses).reduce((sum, val) => sum + val, 0);

    Object.keys(year_expenses).forEach(key => {
        year_expenses[key] = year_expenses[key] / total;
    });

    const used = Object.values(year_expenses).reduce((sum, val) => sum + val, 0);
    year_expenses["Other"] = 1 - used;


    const regions = [
        { name: "Needs", color: barColors["Needs"], portion: year_expenses["Needs"] },
        { name: "Wants", color: barColors["Wants"], portion: year_expenses["Wants"] },
        { name: "Savings", color: barColors["Savings"], portion: year_expenses["Savings"] },
        { name: "Other", color: "gray", portion: year_expenses["Other"] },
    ];

    let x = 0;
    regions.forEach(region => {
        const regionWidth = width * region.portion;
        ctx.fillStyle = region.color;
        ctx.fillRect(x, 0, regionWidth, height);

        ctx.font = "bold 16px Arial";
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillStyle = 'black'


        if (region.portion >= 0.05) {
            ctx.fillText(Math.round(year_expenses[region.name] * 100, 1) + " %", x + regionWidth / 2, canvas.height / 2 + 2);
        }

        x += regionWidth;
    });
});