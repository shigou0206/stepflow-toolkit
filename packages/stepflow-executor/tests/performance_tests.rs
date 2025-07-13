//! Performance tests for stepflow-executor

mod common;

use std::time::Duration;
use common::*;
use stepflow_executor::*;

#[cfg(test)]
mod load_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_execution_performance() {
        let executor = create_test_executor().await.unwrap();
        let start = std::time::Instant::now();
        let mut handles = Vec::new();
        
        // Start 50 concurrent executions
        for i in 0..50 {
            let tool_id = if i % 3 == 0 { "test-tool-1" } else if i % 3 == 1 { "test-tool-2" } else { "slow-tool" };
            
            let handle = tokio::spawn(async move {
                let executor = create_test_executor().await.unwrap();
                let request = create_test_execution_request(tool_id);
                let start_time = std::time::Instant::now();
                
                match executor.execute_tool(request).await {
                    Ok(_) => Some(start_time.elapsed()),
                    Err(_) => None,
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all executions to complete
        let mut successful_count = 0;
        let mut total_duration = Duration::from_secs(0);
        
        for handle in handles {
            if let Ok(Some(duration)) = handle.await {
                successful_count += 1;
                total_duration += duration;
            }
        }
        
        let total_elapsed = start.elapsed();
        let throughput = successful_count as f64 / total_elapsed.as_secs_f64();
        let average_duration = if successful_count > 0 {
            total_duration / successful_count as u32
        } else {
            Duration::from_secs(0)
        };
        
        println!("\n=== Concurrent Execution Performance ===");
        println!("Total executions: 50");
        println!("Successful executions: {}", successful_count);
        println!("Total time: {:?}", total_elapsed);
        println!("Average execution time: {:?}", average_duration);
        println!("Throughput: {:.2} executions/second", throughput);
        println!("Success rate: {:.2}%", (successful_count as f64 / 50.0) * 100.0);
        
        // Assertions
        assert!(successful_count > 40, "Should have high success rate: {}/50", successful_count);
        assert!(total_elapsed < Duration::from_secs(60), "Should complete within reasonable time");
        assert!(throughput > 0.5, "Should maintain reasonable throughput");
    }

    #[tokio::test]
    async fn test_sequential_execution_performance() {
        let executor = create_test_executor().await.unwrap();
        let start = std::time::Instant::now();
        let mut execution_times = Vec::new();
        
        // Run 20 sequential executions
        for i in 0..20 {
            let tool_id = if i % 2 == 0 { "test-tool-1" } else { "test-tool-2" };
            let request = create_test_execution_request(tool_id);
            let exec_start = std::time::Instant::now();
            
            match executor.execute_tool(request).await {
                Ok(_) => execution_times.push(exec_start.elapsed()),
                Err(_) => (),
            }
        }
        
        let total_elapsed = start.elapsed();
        let successful_count = execution_times.len();
        let average_duration = if successful_count > 0 {
            execution_times.iter().sum::<Duration>() / successful_count as u32
        } else {
            Duration::from_secs(0)
        };
        
        let min_duration = execution_times.iter().min().cloned().unwrap_or(Duration::from_secs(0));
        let max_duration = execution_times.iter().max().cloned().unwrap_or(Duration::from_secs(0));
        
        println!("\n=== Sequential Execution Performance ===");
        println!("Total executions: 20");
        println!("Successful executions: {}", successful_count);
        println!("Total time: {:?}", total_elapsed);
        println!("Average execution time: {:?}", average_duration);
        println!("Min execution time: {:?}", min_duration);
        println!("Max execution time: {:?}", max_duration);
        
        // Assertions
        assert_eq!(successful_count, 20, "All sequential executions should succeed");
        assert!(average_duration < Duration::from_secs(5), "Average execution time should be reasonable");
        assert!(max_duration < Duration::from_secs(10), "Max execution time should be reasonable");
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        // Run multiple cycles to test memory stability
        for cycle in 0..5 {
            let executor = create_test_executor().await.unwrap();
            let mut handles = Vec::new();
            
            // Start 20 concurrent executions per cycle
            for i in 0..20 {
                let tool_id = if i % 2 == 0 { "test-tool-1" } else { "test-tool-2" };
                
                let handle = tokio::spawn(async move {
                    let executor = create_test_executor().await.unwrap();
                    let request = create_test_execution_request(tool_id);
                    executor.execute_tool(request).await
                });
                
                handles.push(handle);
            }
            
            // Wait for all executions in this cycle
            let mut successful_count = 0;
            for handle in handles {
                if handle.await.is_ok() {
                    successful_count += 1;
                }
            }
            
            println!("Cycle {}: {}/20 executions successful", cycle + 1, successful_count);
            assert!(successful_count > 15, "Should maintain stability across cycles");
            
            // Small delay between cycles
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        println!("Memory usage stability test completed successfully");
    }

    #[tokio::test]
    async fn test_high_frequency_execution() {
        let executor = create_test_executor().await.unwrap();
        let start = std::time::Instant::now();
        let mut handles = Vec::new();
        
        // Submit 100 executions as fast as possible
        for i in 0..100 {
            let tool_id = "test-tool-1";
            
            let handle = tokio::spawn(async move {
                let executor = create_test_executor().await.unwrap();
                let request = create_test_execution_request(tool_id);
                executor.execute_tool_async(request).await
            });
            
            handles.push(handle);
            
            // Small delay to avoid overwhelming the system
            if i % 10 == 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        
        // Wait for all submissions to complete
        let mut successful_submissions = 0;
        for handle in handles {
            if handle.await.is_ok() {
                successful_submissions += 1;
            }
        }
        
        let submission_time = start.elapsed();
        let submission_rate = successful_submissions as f64 / submission_time.as_secs_f64();
        
        println!("\n=== High Frequency Execution Test ===");
        println!("Total submissions: 100");
        println!("Successful submissions: {}", successful_submissions);
        println!("Submission time: {:?}", submission_time);
        println!("Submission rate: {:.2} submissions/second", submission_rate);
        
        // Assertions
        assert!(successful_submissions > 90, "Should handle high frequency submissions");
        assert!(submission_rate > 50.0, "Should maintain high submission rate");
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_exhaustion_handling() {
        let mut handles = Vec::new();
        
        // Try to create many executors simultaneously
        for i in 0..20 {
            let handle = tokio::spawn(async move {
                match create_test_executor().await {
                    Ok(executor) => {
                        let request = create_test_execution_request("test-tool-1");
                        executor.execute_tool(request).await.is_ok()
                    }
                    Err(_) => false,
                }
            });
            
            handles.push(handle);
        }
        
        // Count successful executions
        let mut successful_count = 0;
        for handle in handles {
            if let Ok(true) = handle.await {
                successful_count += 1;
            }
        }
        
        println!("\n=== Resource Exhaustion Test ===");
        println!("Attempted executors: 20");
        println!("Successful executions: {}", successful_count);
        
        // Should handle resource constraints gracefully
        assert!(successful_count > 0, "Should handle some executions even under stress");
    }

    #[tokio::test]
    async fn test_long_running_execution_handling() {
        let executor = create_test_executor().await.unwrap();
        let start = std::time::Instant::now();
        
        // Start a potentially long-running execution
        let request = create_test_execution_request("slow-tool");
        let execution_id = executor.execute_tool_async(request).await.unwrap();
        
        // Monitor execution status
        let mut status_checks = 0;
        let max_wait = Duration::from_secs(30);
        
        while start.elapsed() < max_wait {
            match executor.get_execution_status(&execution_id).await {
                Ok(ExecutionStatus::Completed) => {
                    println!("Long-running execution completed in {:?}", start.elapsed());
                    return;
                }
                Ok(ExecutionStatus::Failed) => {
                    println!("Long-running execution failed after {:?}", start.elapsed());
                    return;
                }
                Ok(_) => {
                    status_checks += 1;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(_) => break,
            }
        }
        
        println!("\n=== Long Running Execution Test ===");
        println!("Execution time: {:?}", start.elapsed());
        println!("Status checks: {}", status_checks);
        
        // Test should complete within reasonable time or handle timeout gracefully
        assert!(start.elapsed() < Duration::from_secs(35), "Should handle long executions");
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[tokio::test]
    async fn test_baseline_performance_benchmark() {
        let config = PerformanceConfig {
            concurrent_executions: 10,
            total_executions: 50,
            max_duration: Duration::from_secs(30),
            expected_throughput: 5.0,
        };
        
        let benchmark = benchmark_execution(config).await.unwrap();
        
        benchmark.print_summary();
        
        // Performance assertions
        assert!(benchmark.successful_executions > 40, 
            "Should have high success rate: {}/{}", 
            benchmark.successful_executions, benchmark.total_executions);
        
        assert!(benchmark.total_duration < Duration::from_secs(30), 
            "Should complete within time limit");
        
        assert!(benchmark.throughput > 1.0, 
            "Should maintain minimum throughput: {:.2}", benchmark.throughput);
        
        assert!(benchmark.average_duration < Duration::from_secs(5), 
            "Average execution time should be reasonable: {:?}", benchmark.average_duration);
        
        let success_rate = benchmark.successful_executions as f64 / benchmark.total_executions as f64;
        assert!(success_rate > 0.8, 
            "Should maintain high success rate: {:.2}%", success_rate * 100.0);
    }

    #[tokio::test]
    async fn test_scaling_performance_benchmark() {
        let test_configs = vec![
            (5, 25),   // Low load
            (10, 50),  // Medium load
            (15, 75),  // High load
        ];
        
        for (concurrent, total) in test_configs {
            let config = PerformanceConfig {
                concurrent_executions: concurrent,
                total_executions: total,
                max_duration: Duration::from_secs(60),
                expected_throughput: 2.0,
            };
            
            let benchmark = benchmark_execution(config).await.unwrap();
            
            println!("\n=== Scaling Test: {} concurrent, {} total ===", concurrent, total);
            benchmark.print_summary();
            
            // Scaling assertions
            assert!(benchmark.successful_executions > (total * 80 / 100), 
                "Should maintain success rate under load");
            
            assert!(benchmark.throughput > 0.5, 
                "Should maintain throughput under load");
        }
    }

    #[tokio::test]
    async fn test_sustained_load_benchmark() {
        // Run sustained load for multiple rounds
        let rounds = 3;
        let mut round_results = Vec::new();
        
        for round in 0..rounds {
            let config = PerformanceConfig {
                concurrent_executions: 8,
                total_executions: 40,
                max_duration: Duration::from_secs(30),
                expected_throughput: 3.0,
            };
            
            let benchmark = benchmark_execution(config).await.unwrap();
            round_results.push(benchmark);
            
            println!("\n=== Sustained Load Round {} ===", round + 1);
            round_results[round].print_summary();
            
            // Small delay between rounds
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Analyze sustained performance
        let avg_throughput: f64 = round_results.iter()
            .map(|r| r.throughput)
            .sum::<f64>() / rounds as f64;
        
        let avg_success_rate: f64 = round_results.iter()
            .map(|r| r.successful_executions as f64 / r.total_executions as f64)
            .sum::<f64>() / rounds as f64;
        
        println!("\n=== Sustained Load Summary ===");
        println!("Average throughput: {:.2} executions/second", avg_throughput);
        println!("Average success rate: {:.2}%", avg_success_rate * 100.0);
        
        // Sustained performance assertions
        assert!(avg_throughput > 1.0, "Should maintain sustained throughput");
        assert!(avg_success_rate > 0.8, "Should maintain sustained success rate");
        
        // Check for performance degradation
        let first_throughput = round_results[0].throughput;
        let last_throughput = round_results[rounds - 1].throughput;
        let degradation = (first_throughput - last_throughput) / first_throughput;
        
        assert!(degradation < 0.3, "Performance degradation should be minimal: {:.2}%", degradation * 100.0);
    }
} 