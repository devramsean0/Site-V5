# frozen_string_literal: true

class WindowComponent < ViewComponent::Base
  renders_one :body

  def initialize(title:)
    @title = title
  end
end
