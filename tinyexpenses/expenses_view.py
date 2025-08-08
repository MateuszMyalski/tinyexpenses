from flask import render_template, redirect, url_for, jsonify
from flask_login import current_user
from .models.expenses import YearExpensesTotals
from .models.categories import CategoryType
from .extensions import users_db
import calendar
from datetime import datetime


def _sort_monthly_expenses_by_category_types(expenses_by_category, categories):
    result = {ct: {} for ct in CategoryType}
    for category, expenses in expenses_by_category.items():
        for ct in CategoryType:
            if category in categories[ct]:
                result[ct][category] = expenses
                break
    return result


def _calculate_yearly_expenses_stats(
    expenses_by_category_type,
):
    year_totals = {}
    monthly_balance = YearExpensesTotals()
    balance_per_type = {ct: YearExpensesTotals() for ct in CategoryType}

    for ct, categories in expenses_by_category_type.items():
        sign = 1 if ct == CategoryType.INCOME else -1
        for totals in categories.values():
            monthly_balance += totals * sign
            balance_per_type.setdefault(ct, YearExpensesTotals())
            balance_per_type[ct] += totals * sign
            year_totals[ct] = year_totals.get(ct, 0.0) + sum(totals) * sign

    return year_totals, monthly_balance, balance_per_type


def _get_user_or_abort(user_id):
    user = users_db.get(user_id)
    if user is None:
        return None, render_template("error.html", message="User not found.")
    return user, None


def _load_year_data(user, year):
    try:
        report = user.get_year_expenses(year)
    except FileNotFoundError:
        return None, None, redirect(url_for("main.expenses_create", year=year))

    try:
        categories = user.get_year_categories(year)
    except FileNotFoundError:
        return None, None, redirect(url_for("main.categories_create", year=year))

    return report, categories, None


def _complete_missing_categories(data, categories):
    for record in categories.get_categories():
        data.setdefault(record.category, YearExpensesTotals())


def _prepare_context(user, report, categories, year):
    data = report.get_expenses_by_category_monthly_totals()
    _complete_missing_categories(data, categories)
    grouped = _sort_monthly_expenses_by_category_types(data, categories)
    year_totals, monthly_balance, balance_per_type = _calculate_yearly_expenses_stats(
        grouped
    )

    return {
        "expenses_by_type": grouped,
        "year_totals": year_totals,
        "monthly_balance": monthly_balance,
        "balance_per_type": balance_per_type,
        "current_balance": report.initial_balance + sum(monthly_balance),
        "currency": user.currency,
    }


def expenses_view_year_get(year: int):
    user, error = _get_user_or_abort(current_user.id)
    if error:
        return error

    report, categories, redirect_response = _load_year_data(user, year)
    if redirect_response:
        return redirect_response

    context = _prepare_context(user, report, categories, year)
    return render_template(
        "expenses_view_year.html",
        year=year,
        expenses_monthly_totals_by_category_type=context["expenses_by_type"],
        year_totals=context["year_totals"],
        monthly_balance=context["monthly_balance"],
        monthly_balance_per_category_type=context["balance_per_type"],
        currency=context["currency"],
        title=f"{year} expenses",
        available_years=user.get_available_expenses_files(),
        current_balance=context["current_balance"],
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:],
    )


def expenses_view_month_get(year: int, month: int):
    user, error = _get_user_or_abort(current_user.id)
    if error:
        return error

    if month not in range(1, len(calendar.month_name)):
        return render_template("error.html", message="Invalid month."), 400

    report, categories, redirect_response = _load_year_data(user, year)
    if redirect_response:
        return redirect_response

    context = _prepare_context(user, report, categories, year)
    return render_template(
        "expenses_view_month.html",
        year=year,
        month=month - 1,
        expenses_monthly_totals_by_category_type=context["expenses_by_type"],
        year_totals=context["year_totals"],
        monthly_balance=context["monthly_balance"],
        monthly_balance_per_category_type=context["balance_per_type"],
        currency=context["currency"],
        title=f"{month}/{year} expenses",
        available_years=user.get_available_expenses_files(),
        current_balance=context["current_balance"],
        CategoryType=CategoryType,
        month_names=calendar.month_name[1:],
    )


def expenses_view_balance_api_get(username, year):
    user, error = _get_user_or_abort(username)
    if error:
        return jsonify({"status": "Unauthorized"}), 401

    try:
        year = int(year)
    except Exception:
        year = datetime.now().date().year

    try:
        report = user.get_year_expenses(year)
        categories = user.get_year_categories(year)
    except Exception:
        return jsonify(
            {"status": f"Could not read expenses or categories for year {year}."}
        ), 500

    context = _prepare_context(user, report, categories, year)
    return jsonify(
        {"status": "Ok", "balance": round(float(context["current_balance"]), 2)}
    ), 200
