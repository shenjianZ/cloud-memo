  $baseUrl = "http://localhost:3000/auth/register"

  $testCases = @(
      @{Name="Windows"; Email="win5@test.com"; DeviceId="desktop-windows-550e8400-e29b-41d4-a716-446655440000"},
      @{Name="macOS"; Email="mac5@test.com"; DeviceId="desktop-macos-650e8400-e29b-41d4-a716-446655440001"},
      @{Name="iPhone"; Email="iphone5@test.com"; DeviceId="mobile-ios-750e8400-e29b-41d4-a716-446655440002"},
      @{Name="iPad"; Email="ipad5@test.com"; DeviceId="mobile-ios-850e8400-e29b-41d4-a716-446655440003"},
      @{Name="Android Phone"; Email="android5@test.com"; DeviceId="mobile-android-950e8400-e29b-41d4-a716-446655440004"},
      @{Name="Android Tablet"; Email="atablet5@test.com"; DeviceId="mobile-android-a50e8400-e29b-41d4-a716-446655440005"}
  )

  $uaMap = @{
      "Windows" = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
      "macOS" = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15'
      "iPhone" = 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15'
      "iPad" = 'Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15'
      "Android Phone" = 'Mozilla/5.0 (Linux; Android 13; SM-S908B) AppleWebKit/537.36 Mobile Safari/537.36'
      "Android Tablet" = 'Mozilla/5.0 (Linux; Android 13; SM-X900) AppleWebKit/537.36 Safari/537.36'
  }

  foreach ($test in $testCases) {
      Write-Host "`n=== 测试: $($test.Name) ===" -ForegroundColor Cyan

      # 手动构建 JSON 字符串
      $jsonBody = "{`"email`":`"$($test.Email)`",`"password`":`"Test123456`",`"device_id`":`"$($test.DeviceId)`"}"
      $userAgent = $uaMap[$test.Name]

      Write-Host "发送 device_id: $($test.DeviceId)" -ForegroundColor Gray
      Write-Host "User-Agent: $userAgent" -ForegroundColor Gray

      try {
          $response = Invoke-RestMethod -Uri $baseUrl -Method POST `
              -Headers @{
                  "Content-Type" = "application/json"
                  "User-Agent" = $userAgent
              } `
              -Body $jsonBody

          Write-Host "✅ 注册成功: $($response.email)" -ForegroundColor Green
          Write-Host "   返回 Device ID: $($response.device_id)" -ForegroundColor Cyan

          # 关键对比：检查返回的 device_id 是否被更新
          if ($test.DeviceId -ne $response.device_id) {
              Write-Host "   ⚠️  device_id 已更新！" -ForegroundColor Yellow
              Write-Host "      原始: $($test.DeviceId)" -ForegroundColor Gray
              Write-Host "      最终: $($response.device_id)" -ForegroundColor Gray
          }
      } catch {
          Write-Host "❌ 失败: $($_.Exception.Message)" -ForegroundColor Red
          if ($_.ErrorDetails) {
              Write-Host "   详细: $($_.ErrorDetails.Message)" -ForegroundColor Red
          }
      }
  }
