from flask import render_template, redirect, url_for
from flask_login import current_user
from .models.accounts import AppUser
from .extensions import users_db
from .models.categories import YearCategories, CategoryRecord
from .csv_edit import render_csv_data_edit_form, handle_csv_data_edit


def _store_categories_data_cb(ctx: dict, data: str):
    categories = [CategoryRecord(*row) for row in data]

    YearCategories.store(ctx["db_file"], categories)


def categories_edit_post(year: int):
    requested_user = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    db_file = requested_user._get_year_categories_file(year)
    if not db_file.exists():
        return redirect(url_for("main.categories_create", year=year))
    return handle_csv_data_edit(
        url_for("main.categories_edit", year=year),
        _store_categories_data_cb,
        {"db_file": db_file},
    )


def categories_edit_get(year: int):
    requested_user: AppUser | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    try:
        year_categories = requested_user.get_year_categories(year)
    except FileNotFoundError:
        return redirect(url_for("main.categories_create", year=year))

    return render_csv_data_edit_form(
        [col.label for col in CategoryRecord.Columns], year_categories.get_categories()
    )
