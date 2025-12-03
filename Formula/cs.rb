class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.9/cs-darwin-amd64"
  sha256 "e413b495e3eb048f489989b548dcf763052a00dd4e0698e9a74ee055a915dcce"
  version "0.1.9"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
