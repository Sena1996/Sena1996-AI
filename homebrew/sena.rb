class Sena < Formula
  desc "SENA - Make Your AI Collaborative and Smarter"
  homepage "https://github.com/Sena1996/Sena1996-AI"
  version "11.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Sena1996/Sena1996-AI/releases/download/v#{version}/sena-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    end
    on_intel do
      url "https://github.com/Sena1996/Sena1996-AI/releases/download/v#{version}/sena-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X64"
    end
  end

  on_linux do
    url "https://github.com/Sena1996/Sena1996-AI/releases/download/v#{version}/sena-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "sena"
  end

  test do
    assert_match "sena", shell_output("#{bin}/sena --version")
  end
end
