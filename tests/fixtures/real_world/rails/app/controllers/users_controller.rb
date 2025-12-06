class UsersController < ApplicationController
  def activate
    # ... logic ...
    flash[:success] = I18n.t("activation.activated")
    redirect_to login_path
  end

  def show
    user = User.find(params[:id])
    if user.avatar.nil?
      render json: { error: I18n.t("avatar.missing") }, status: 404
      return
    end
  end
  
  def check_settings
    unless SiteSetting.exists?(name: params[:setting])
      # Dynamic interpolation example
      render_json_error I18n.t("site_setting_missing", name: params[:setting]), status: 500
    end
  end
end
