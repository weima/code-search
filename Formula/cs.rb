class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.2/cs-darwin-amd64"
  sha256 "03950774584ed8b8439eac574e218ae3846323d2ce87bea09c8750ce5c6b7957"
  version "0.1.2"
  license "Apache-2.0"

  def install
    bin.install "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
