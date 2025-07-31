from flask import render_template, get_flashed_messages, flash, redirect
from flask_wtf import FlaskForm
from wtforms import SubmitField, HiddenField
import json

CSV_EDITOR_FLASH_LABEL = "csv_editor_info"


class FileEditForm(FlaskForm):
    table_data = HiddenField("Table Data")
    submit = SubmitField("Save")


def handle_csv_data_edit(
    redirect_on_success_url: str, store_data_cb: callable, ctx: dict
):
    form = FileEditForm()

    if not form.validate_on_submit():
        return render_template(
            "csv_edit.html",
            form=form,
            infos=[("error", "Request could not be validated.")],
        )

    table_data = json.loads(form.table_data.data)
    store_data_cb(ctx, table_data)

    flash("File edited successfully!", CSV_EDITOR_FLASH_LABEL)
    return redirect(redirect_on_success_url)


def render_csv_data_edit_form(
    col_labels: list[str],
    csv_content: iter,
):
    form = FileEditForm()

    flashed_info = [
        ("info", msg) for msg in get_flashed_messages(False, CSV_EDITOR_FLASH_LABEL)
    ]

    return render_template(
        "csv_edit.html",
        form=form,
        col_labels=col_labels,
        content=csv_content,
        infos=flashed_info,
    )
