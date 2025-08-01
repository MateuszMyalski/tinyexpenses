from flask import render_template, flash, get_flashed_messages, request
from flask_login import current_user
from flask_wtf import FlaskForm
from wtforms import StringField, PasswordField, SubmitField
from wtforms import validators
from .models.accounts import User
from .extensions import users_db
from .token import generate_user_token

ACCOUNT_APPEND_FLASH_INFO_LABEL = "account_info_info"
ACCOUNT_APPEND_FLASH_ERROR_LABEL = "account_error_info"


class UserDetailsForm(FlaskForm):
    full_name = StringField("Full name", validators=[validators.DataRequired()])
    currency = StringField(
        "Currency",
        validators=[
            validators.DataRequired(),
            validators.Length(
                min=1, max=3, message="The currency must be between 1-3 symbols"
            ),
        ],
    )
    submit = SubmitField("Update details")

    def populate_default_full_name(self, full_name):
        self.full_name.data = full_name
        self.full_name.default = full_name

    def populate_default_currency(self, currency):
        self.currency.data = currency
        self.currency.default = currency


class XApiKeyGenerateForm(FlaskForm):
    token = StringField(
        "X-Api-Key Token", validators=[validators.ReadOnly()], default="First generate...", 
    )
    submit = SubmitField("Generate x-api-key")


class ChangePasswordForm(FlaskForm):
    current_passw = PasswordField(
        "Current password", validators=[validators.DataRequired()]
    )
    new_passw = PasswordField("New password", validators=[validators.DataRequired()])
    confirm = PasswordField(
        "Confirm new password",
        validators=[
            validators.DataRequired(),
            validators.EqualTo("new_passw", message="Passwords must match"),
        ],
    )
    submit = SubmitField("Change password")


def _collect_flash_messages():
    infos = [
        ("info", msg)
        for msg in get_flashed_messages(False, ACCOUNT_APPEND_FLASH_INFO_LABEL)
    ]

    errors = [
        ("error", msg)
        for msg in get_flashed_messages(False, ACCOUNT_APPEND_FLASH_ERROR_LABEL)
    ]

    return [*infos, *errors]


def _handle_details_change(user: User, form: UserDetailsForm) -> None:
    if not form.validate_on_submit():
        flash("Request could not be validated.", ACCOUNT_APPEND_FLASH_ERROR_LABEL)

    user.set_full_name(form.full_name.data)
    user.set_currency(form.currency.data)

    flash("Details changed.", ACCOUNT_APPEND_FLASH_INFO_LABEL)


def _handle_token_generation(user: User, form: XApiKeyGenerateForm) -> None:
    if not form.validate_on_submit():
        flash("Request could not be validated.", ACCOUNT_APPEND_FLASH_ERROR_LABEL)

    token = user.set_token()
    user_token = generate_user_token(token)
    
    form.token.default = user_token
    form.token.data = user_token

    flash("Token generated.", ACCOUNT_APPEND_FLASH_INFO_LABEL)


def _handle_password_change(user: User, form: ChangePasswordForm) -> None:
    if not form.validate_on_submit():
        flash("Request could not be validated.", ACCOUNT_APPEND_FLASH_ERROR_LABEL)

    success = user.set_password(
        current=form.current_passw.data, new=form.new_passw.data
    )

    if success:
        flash("Password changed.", ACCOUNT_APPEND_FLASH_INFO_LABEL)
    else:
        flash("Invalid password.", ACCOUNT_APPEND_FLASH_ERROR_LABEL)


def account_post():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    form_details = UserDetailsForm(prefix="details")
    form_change_password = ChangePasswordForm(prefix="password")
    form_api_token = XApiKeyGenerateForm(prefix="token")

    if form_details.submit.data:
        _handle_details_change(requested_user, form_details)

    if form_change_password.submit.data:
        _handle_password_change(requested_user, form_change_password)

    if form_api_token.submit.data:
        _handle_token_generation(requested_user, form_api_token)

    form_details.populate_default_currency(requested_user.currency)
    form_details.populate_default_full_name(requested_user.full_name)

    return render_template(
        "account_view.html",
        form_details=form_details,
        form_change_password=form_change_password,
        username=requested_user.username,
        form_api_token=form_api_token,
        host_url=request.host_url,
        infos=_collect_flash_messages(),
    )


def account_get():
    requested_user: User | None = users_db.get(current_user.id)

    if requested_user is None:
        return render_template("error.html", message="User not found.")

    form_details = UserDetailsForm(prefix="details")
    form_details.populate_default_currency(requested_user.currency)
    form_details.populate_default_full_name(requested_user.full_name)

    form_change_password = ChangePasswordForm(prefix="password")

    form_api_token = XApiKeyGenerateForm(prefix="token")

    return render_template(
        "account_view.html",
        form_details=form_details,
        form_change_password=form_change_password,
        username=requested_user.username,
        form_api_token=form_api_token,
        host_url=request.host_url,
        infos=_collect_flash_messages(),
    )
