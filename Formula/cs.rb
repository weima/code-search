class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.1.8/cs-darwin-amd64"
  sha256 "de2bdfbede7fc6fb701e7ea6d84df3a7987f7d856f246597cd54bd77c0dfefcf"
  version "0.1.8"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
