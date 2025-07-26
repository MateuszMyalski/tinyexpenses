from flask import render_template
from flask_login import current_user
from .models import YearExpensesReport, Categories
from .extensions import users_db
import calendar

def view_month_display(year: int, month: int):
    requested_user = users_db.get(current_user.id)
    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        year = int(year)
        month = int(month)

        year_report = YearExpensesReport(requested_user)
        year_report.load_expenses(year)

        categories = Categories(requested_user)
        categories.load_categories()
    except (FileNotFoundError, ValueError) as e:
        return render_template("error.html", message=str(e))

    # Miesiące z calendar, żeby wiedzieć który indeks odpowiada miesiącowi (1-based)
    months = [i for i in range(len(calendar.month_name) - 1)]

    if month not in months:
        return render_template("error.html", message=f"Invalid month: {month}")

    expenses_by_category_all = year_report.get_expenses_by_category()

    expenses_by_category = {}
    for category, monthly_values in expenses_by_category_all.items():
        if len(monthly_values) >= month:
            expenses_by_category[category] = monthly_values[month]
        else:
            expenses_by_category[category] = 0

    for cat in categories.get_categories():
        if cat not in expenses_by_category:
            expenses_by_category[cat] = 0

    total_expenses = exp_calcs_sum_months_total(year_report, categories)[month]

    current_balance = exp_calc_current_balance(year_report, categories)

    available_years = requested_user.get_available_expenses_year_reports()

    return render_template(
        "view_month.html",
        year=year,
        month=month,
        categories=categories.get_categories_by_type(),
        expenses=expenses_by_category,
        total_expenses=total_expenses,
        currency=requested_user.currency,
        title=f"Expenses for {month}/{year}",
        available_years=available_years,
        months=list(calendar.month_name)[1:],
        current_balance=current_balance,
    )
