#!/usr/bin/env ruby
require 'json'

input = ARGF.read.lines
input.shift
input.sort!
input.uniq!

agents = {}

input.each do |line|
  match = line.match(/^"bundler\/(?<bundler>[a-z0-9.]+) rubygems\/(?<rubygems>[a-z0-9.]+) ruby\/(?<ruby>[a-z0-9.]+) \((?<cpu>[^-]+)-(?<vendor>[^-]+)-(?<os>.+)\) command\/(?<command>.*?) (jruby\/(?<jruby>[a-z0-9.]+) )?(options\/(?<options>.+?) )?(ci\/(?<ci>.+) )?[a-f0-9]+( (?<extra>.+))?"$/)

  if match.nil?
    puts "No match! Line was:\n#{line}"
    next
  end

  info = match.named_captures
  options = info.delete("options")

  options.split(",").each do |name|
    agents["options"] ||= Hash.new(0)
    agents["options"][name] += 1
  end if options

  info.each do |name, value|
    next if value.nil?
    agents[name] ||= Hash.new(0)
    agents[name][value] += 1
  end
end

agents["platforms"] = Hash.new(0)
agents["os"].each do |k, v|
  case k
  when /darwin/
    agents["platforms"]["darwin"] += v
  when /linux/
    agents["platforms"]["linux"] += v
  when /mswin|mingw/
    agents["platforms"]["windows"] += v
  when /bsd/
    agents["platforms"]["bsd"] += v
  end
end

puts JSON.pretty_generate(agents)
