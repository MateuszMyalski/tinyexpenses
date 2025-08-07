from flask import render_template, flash, redirect
from flask_wtf import FlaskForm
from wtforms import SubmitField, HiddenField
from .models.flash import FlashType, flash_collect
import json


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

    flash("File edited successfully!", FlashType.INFO.name)
    return redirect(redirect_on_success_url)


def render_csv_data_edit_form(
    col_labels: list[str],
    csv_content: iter,
):
    form = FileEditForm()

    return render_template(
        "csv_edit.html",
        form=form,
        col_labels=col_labels,
        content=csv_content,
        infos=flash_collect(),
    )
