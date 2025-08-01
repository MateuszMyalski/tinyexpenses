from flask import render_template, redirect, url_for
from flask_login import current_user
from .models.accounts import User
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

    year_categories_file = requested_user.get_categories_file(year)

    if not year_categories_file.exists():
        return redirect(url_for("main.categories_create", year=year))

    return handle_csv_data_edit(
        url_for("main.categories_edit", year=year),
        _store_categories_data_cb,
        {"db_file": year_categories_file},
    )


def categories_edit_get(year: int):
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    year_categories_file = requested_user.get_categories_file(year)

    if not year_categories_file.exists():
        return redirect(url_for("main.categories_create", year=year))

    year_categories: list[CategoryRecord] = YearCategories(
        year_categories_file
    ).get_categories()

    return render_csv_data_edit_form(
        [col.label for col in CategoryRecord.Columns], year_categories
    )
