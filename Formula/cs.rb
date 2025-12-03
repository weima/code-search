class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.10/cs-darwin-amd64"
  sha256 "039436511ba5b740ad44c487c3e4ec7eef395b89d7b7cd2ba77bc7f214d4f265"
  version "0.1.10"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
