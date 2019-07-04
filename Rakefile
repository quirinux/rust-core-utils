
Signal.trap("INT") do
  exit
end

Signal.trap("TERM") do
  exit
end

def cargo_build(mode = "")
  mode = "--#{mode.to_s}" unless mode.nil?
  system "cargo build #{mode}" or exit!(1)
end

def cmd_args
  if ARGV.include? '--'
    ARGV
      .join(' ')
      .split('--', 2)
      .last
  end
end

def run(mode, bin)
  cmd = ['.', 'target', mode, bin].map{|m| m.to_s}.join('/')
  cmd = [cmd, cmd_args].join(' ')
  puts cmd
  system cmd
end

namespace :build do
  desc 'Build debug'
  task :debug, [:run] do |task, args|
    cargo_build
    Rake::Task[:run].invoke(args[:run])
  end

  desc 'Build release'
  task :release do |task, args|
    cargo_build :release
    Rake::Task[:run].invoke(args[:run], :release)
  end

  desc 'Build debug and release'
  task all: [:debug, :release]
end


desc 'Run bin'
task :run, [:run, :mode] do |task, args|
  mode = args[:mode] || :debug
  bin = args[:run]
  run(mode, bin) unless bin.nil?
end


