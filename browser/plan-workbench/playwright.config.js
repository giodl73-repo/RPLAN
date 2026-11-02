import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'tests',testMatch:'*.spec.js',workers:1,timeout:30000,
  use:{baseURL:process.env.RPLAN_URL??'http://127.0.0.1:8774/RPLAN/',headless:true,trace:'retain-on-failure',screenshot:'only-on-failure'},
  webServer:process.env.RPLAN_URL?undefined:{command:'python -m http.server 8774 --directory ../../target/plan-prototype --bind 127.0.0.1',url:'http://127.0.0.1:8774/RPLAN/',reuseExistingServer:false},
});
