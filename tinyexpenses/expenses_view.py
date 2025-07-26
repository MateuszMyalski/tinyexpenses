from flask import render_template
from flask_login import current_user
from .models import YearExpensesReport, Categories, CategoryType, YearExpensesTotals
from .extensions import users_db
import calendar


def sort_monthly_expenses_by_category_types(
    expenses_by_category: dict[str, YearExpensesTotals], categories: Categories
) -> dict[CategoryType, dict[str, YearExpensesTotals]]:
    result = {ct: {} for ct in CategoryType}
    for category, expenses_by_month in expenses_by_category.items():
        if category in categories.get_categories_income():
            result[CategoryType.INCOME][category] = expenses_by_month
        elif category in categories.get_categories_needs():
            result[CategoryType.NEEDS][category] = expenses_by_month
        elif category in categories.get_categories_savings():
            result[CategoryType.SAVINGS][category] = expenses_by_month
        elif category in categories.get_categories_wants():
            result[CategoryType.WANTS][category] = expenses_by_month
    return result


def calculate_yearly_expenses_stats(
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


def expenses_view_year(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        year_report = YearExpensesReport(requested_user, int(year))
        categories = Categories(requested_user)
    except (FileNotFoundError, ValueError) as e:
        return render_template("error.html", message=str(e))

    expenses_monthly_totals_by_category = (
        year_report.get_expenses_by_category_monthly_totals()
    )

    # Ensure all categories are present in expenses_by_category, missing ones set to 0
    for cat in categories.get_categories():
        if cat not in expenses_monthly_totals_by_category:
            expenses_monthly_totals_by_category[cat] = YearExpensesTotals()

    expenses_monthly_totals_by_category_type: dict[
        CategoryType, dict[str, YearExpensesTotals]
    ] = sort_monthly_expenses_by_category_types(
        expenses_monthly_totals_by_category, categories
    )

    year_totals, monthly_balance, monthly_balance_per_category_type = (
        calculate_yearly_expenses_stats(expenses_monthly_totals_by_category_type)
    )

    return render_template(
        "view_year.html",
        year=year,
        expenses_monthly_totals_by_category_type=expenses_monthly_totals_by_category_type,
        year_totals=year_totals,
        monthly_balance=monthly_balance,
        monthly_balance_per_category_type=monthly_balance_per_category_type,
        currency=requested_user.currency,
        title=f"{year} expenses",
        available_years=requested_user.get_available_expenses_year_reports(),
        current_balance=year_report.initial_balance + sum(monthly_balance),
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:]
    )


def expenses_view_month(year: int, month: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        month = int(month)

        if month not in range(1, len(calendar.month_name)):
            raise ValueError(f"Invalid month number {month}")

        year_report = YearExpensesReport(requested_user, int(year))
        categories = Categories(requested_user)
    except (FileNotFoundError, ValueError) as e:
        return render_template("error.html", message=str(e))

    expenses_monthly_totals_by_category = (
        year_report.get_expenses_by_category_monthly_totals()
    )

    # Ensure all categories are present in expenses_by_category, missing ones set to 0
    for cat in categories.get_categories():
        if cat not in expenses_monthly_totals_by_category:
            expenses_monthly_totals_by_category[cat] = YearExpensesTotals()

    expenses_monthly_totals_by_category_type: dict[
        CategoryType, dict[str, YearExpensesTotals]
    ] = sort_monthly_expenses_by_category_types(
        expenses_monthly_totals_by_category, categories
    )

    year_totals, monthly_balance, monthly_balance_per_category_type = (
        calculate_yearly_expenses_stats(expenses_monthly_totals_by_category_type)
    )

    return render_template(
        "view_month.html",
        year=year,
        month=month - 1,
        expenses_monthly_totals_by_category_type=expenses_monthly_totals_by_category_type,
        year_totals=year_totals,
        monthly_balance=monthly_balance,
        monthly_balance_per_category_type=monthly_balance_per_category_type,
        currency=requested_user.currency,
        title=f"{month}/{year} expenses",
        available_years=requested_user.get_available_expenses_year_reports(),
        current_balance=year_report.initial_balance + sum(monthly_balance),
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:]
    )
