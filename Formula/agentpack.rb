class Agentpack < Formula
  desc "Dependency manager for MCP servers and AI agents"
  homepage "https://github.com/ricardosalcedo/agentpack"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/ricardosalcedo/agentpack/releases/download/v#{version}/agentpack-darwin-arm64"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/ricardosalcedo/agentpack/releases/download/v#{version}/agentpack-darwin-amd64"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/ricardosalcedo/agentpack/releases/download/v#{version}/agentpack-linux-arm64"
      sha256 "PLACEHOLDER"
    else
      url "https://github.com/ricardosalcedo/agentpack/releases/download/v#{version}/agentpack-linux-amd64"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    binary = Dir["agentpack-*"].first || "agentpack"
    bin.install binary => "agentpack"
  end

  test do
    assert_match "agentpack", shell_output("#{bin}/agentpack --version")
  end
end
