from flask import render_template, redirect, url_for
from flask_login import current_user
from .models.expenses import YearExpensesReport, YearExpensesTotals
from .models.categories import CategoryType, YearCategories
from .extensions import users_db
import calendar


def _sort_monthly_expenses_by_category_types(
    expenses_by_category: dict[str, YearExpensesTotals], categories: YearCategories
) -> dict[CategoryType, dict[str, YearExpensesTotals]]:
    result = {ct: {} for ct in CategoryType}

    for category, expenses_by_month in expenses_by_category.items():
        if category in categories[CategoryType.INCOME]:
            result[CategoryType.INCOME][category] = expenses_by_month
        elif category in categories[CategoryType.NEEDS]:
            result[CategoryType.NEEDS][category] = expenses_by_month
        elif category in categories[CategoryType.SAVINGS]:
            result[CategoryType.SAVINGS][category] = expenses_by_month
        elif category in categories[CategoryType.WANTS]:
            result[CategoryType.WANTS][category] = expenses_by_month

    return result


def _calculate_yearly_expenses_stats(
    expenses_monthly_totals_by_category_type: dict[
        CategoryType, dict[str, YearExpensesTotals]
    ],
) -> tuple[dict, YearExpensesTotals, dict]:
    year_totals = {}
    monthly_balance = YearExpensesTotals()
    monthly_balance_per_category_type = {}

    for category_type, categories in expenses_monthly_totals_by_category_type.items():
        sign: int = 1 if category_type == CategoryType.INCOME else -1
        for _, totals in categories.items():
            monthly_balance += totals * sign

            monthly_balance_per_category_type.setdefault(
                category_type, YearExpensesTotals()
            )
            monthly_balance_per_category_type[category_type] += totals * sign

            year_totals.setdefault(category_type, float())
            year_totals[category_type] += sum(totals) * sign

    return year_totals, monthly_balance, monthly_balance_per_category_type


def expenses_view_year_display(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        year_expenses_file = requested_user.get_expenses_report_file(year)
        year_expense = YearExpensesReport(year_expenses_file)
    except FileNotFoundError as _:
        return redirect(url_for("main.expenses_create", year=year))

    try:
        year_categories_file = requested_user.get_categories_file(year)
        year_categories = YearCategories(year_categories_file)
    except FileNotFoundError as _:
        return redirect(url_for("main.categories_create", year=year))

    expenses_monthly_totals_by_category = (
        year_expense.get_expenses_by_category_monthly_totals()
    )

    # Ensure all categories are present in expenses_by_category, missing ones set to 0
    for category_record in year_categories.get_categories():
        if category_record.category not in expenses_monthly_totals_by_category:
            expenses_monthly_totals_by_category[category_record.category] = (
                YearExpensesTotals()
            )

    expenses_monthly_totals_by_category_type: dict[
        CategoryType, dict[str, YearExpensesTotals]
    ] = _sort_monthly_expenses_by_category_types(
        expenses_monthly_totals_by_category, year_categories
    )

    year_totals, monthly_balance, monthly_balance_per_category_type = (
        _calculate_yearly_expenses_stats(expenses_monthly_totals_by_category_type)
    )

    return render_template(
        "expenses_view_year.html",
        year=year,
        expenses_monthly_totals_by_category_type=expenses_monthly_totals_by_category_type,
        year_totals=year_totals,
        monthly_balance=monthly_balance,
        monthly_balance_per_category_type=monthly_balance_per_category_type,
        currency=requested_user.currency,
        title=f"{year} expenses",
        available_years=requested_user.get_available_expenses_reports(),
        current_balance=year_expense.initial_balance + sum(monthly_balance),
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:],
    )


def expenses_view_month_display(year: int, month: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        if int(month) not in range(1, len(calendar.month_name)):
            raise ValueError(f"Invalid month number {month}")

        year_expenses_file = requested_user.get_expenses_report_file(year)
        year_report = YearExpensesReport(year_expenses_file)
    except FileNotFoundError as _:
        return redirect(url_for("main.expenses_create", year=year))

    try:
        year_categories_file = requested_user.get_categories_file(year)
        year_categories = YearCategories(year_categories_file)
    except FileNotFoundError as _:
        return redirect(url_for("main.categories_create", year=year))

    expenses_monthly_totals_by_category = (
        year_report.get_expenses_by_category_monthly_totals()
    )

    # Ensure all categories are present in expenses_by_category, missing ones set to 0
    for category_record in year_categories.get_categories():
        if category_record.category not in expenses_monthly_totals_by_category:
            expenses_monthly_totals_by_category[category_record.category] = (
                YearExpensesTotals()
            )

    expenses_monthly_totals_by_category_type: dict[
        CategoryType, dict[str, YearExpensesTotals]
    ] = _sort_monthly_expenses_by_category_types(
        expenses_monthly_totals_by_category, year_categories
    )

    year_totals, monthly_balance, monthly_balance_per_category_type = (
        _calculate_yearly_expenses_stats(expenses_monthly_totals_by_category_type)
    )

    return render_template(
        "expenses_view_month.html",
        year=year,
        month=int(month) - 1,
        expenses_monthly_totals_by_category_type=expenses_monthly_totals_by_category_type,
        year_totals=year_totals,
        monthly_balance=monthly_balance,
        monthly_balance_per_category_type=monthly_balance_per_category_type,
        currency=requested_user.currency,
        title=f"{month}/{year} expenses",
        available_years=requested_user.get_available_expenses_reports(),
        current_balance=year_report.initial_balance + sum(monthly_balance),
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:],
    )
