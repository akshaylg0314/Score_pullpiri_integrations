// Utility for managing dashboard refresh when switching between dashboards
export const DashboardRefreshManager = {
  // Set current active dashboard
  setActiveDashboard: (dashboardName) => {
    localStorage.setItem('activeDashboard', dashboardName);
    localStorage.setItem('dashboardSwitchTime', Date.now().toString());
  },

  // Check if dashboard should refresh based on switch detection
  shouldRefreshDashboard: (dashboardName) => {
    const lastActive = localStorage.getItem('activeDashboard');
    const switchTime = localStorage.getItem('dashboardSwitchTime');
    
    // If different dashboard was active, or no timestamp, refresh
    if (!lastActive || !switchTime || lastActive !== dashboardName) {
      return true;
    }

    // If more than 5 seconds since last switch, refresh
    const timeSinceSwitch = Date.now() - parseInt(switchTime);
    return timeSinceSwitch > 5000;
  },

  // Force refresh flag for current dashboard
  forceRefresh: (dashboardName) => {
    localStorage.setItem(`${dashboardName}_forceRefresh`, 'true');
  },

  // Check and clear force refresh flag
  checkAndClearForceRefresh: (dashboardName) => {
    const shouldRefresh = localStorage.getItem(`${dashboardName}_forceRefresh`) === 'true';
    if (shouldRefresh) {
      localStorage.removeItem(`${dashboardName}_forceRefresh`);
    }
    return shouldRefresh;
  }
};