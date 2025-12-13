class Cs < Formula
  desc "Intelligent code search tool for tracing text to implementation code"
  homepage "https://github.com/weima/code-search"
  url "https://github.com/weima/code-search/releases/download/v0.3.2/cs-darwin-amd64"
  sha256 "bf2b45976aeef2473d06b5ba16281dc2baab5495bd2facce9f6fcfba1d08045b"
  version "0.3.2"
  license "Apache-2.0"

  def install
    bin.install "cs-darwin-amd64" => "cs"
  end

  test do
    system "#{bin}/cs", "--help"
  end
end
