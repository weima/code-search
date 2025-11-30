class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.0/code-search-cli-v0.1.0-x86_64-apple-darwin.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  version "0.1.0"
  license "Apache-2.0"

  def install
    bin.install "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
