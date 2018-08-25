require "aws-sdk-s3"
require "pathname"

# This script will download SAMPLE_COUNT gzipped log files, into the
# DESTINATION directory, with filenames starting on START_DATE through today.

DESTINATION = "sample_logs".freeze
SAMPLE_COUNT = 100
START_DATE = "2017-06-04".freeze

s3 = Aws::S3::Client.new

sample_logs = Pathname.new(DESTINATION)
sample_logs.mkpath

log_date = Date.parse(START_DATE)
jump_size = (log_date..Date.today).count / SAMPLE_COUNT

until log_date > Date.today
  logs = s3.list_objects(
    bucket: "rubygems-logs.rubytogether",
    prefix: "fastly_json/#{log_date.iso8601}"
  )
  puts "#{log_date.iso8601} had #{logs.contents.size} logfiles"

  chosen_log = logs.contents.sample
  log_filename = chosen_log.key.gsub(%r{^fastly_json/}, "")

  puts "Downloading #{log_filename}..."
  s3.get_object(
    bucket: "rubygems-logs.rubytogether",
    key: chosen_log.key,
    response_target: sample_logs.join(log_filename)
  )

  log_date += jump_size
  puts
end
