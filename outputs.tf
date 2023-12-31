output "lambda_invoke_arn" {
  value = aws_lambda_function.invoke.qualified_arn
}

output "lambda_load_arn" {
  value = aws_lambda_function.load.qualified_arn
}

output "sns_topic_rdf_load_arn" {
  value = aws_sns_topic.rdf_load.arn
}

output "bucket_domain_name" {
  value = aws_s3_bucket.source_data.bucket_domain_name
}

output "bucket_id" {
  value = aws_s3_bucket.source_data.id
}

output "bucket_arn" {
  value = aws_s3_bucket.source_data.arn
}
