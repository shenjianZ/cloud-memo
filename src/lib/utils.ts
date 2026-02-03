import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 检测是否为移动端设备
 */
export function isMobileDevice(): boolean {
  if (typeof window === 'undefined') return false;
  
  const userAgent = navigator.userAgent || navigator.vendor || (window as any).opera;
  
  // 检测移动设备的关键字
  const mobileRegex = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i;
  
  // 检测屏幕尺寸
  const isSmallScreen = window.innerWidth <= 768;
  
  return mobileRegex.test(userAgent) || isSmallScreen;
}

/**
 * 检测是否为平板设备
 */
export function isTablet(): boolean {
  if (typeof window === 'undefined') return false;
  
  const userAgent = navigator.userAgent || navigator.vendor || (window as any).opera;
  
  // iPad 检测
  const isIPad = /iPad/i.test(userAgent);
  
  // Android 平板检测
  const isAndroidTablet = /Android/i.test(userAgent) && !/Mobile/i.test(userAgent);
  
  // 屏幕尺寸检测（介于手机和平板之间）
  const isMediumScreen = window.innerWidth > 768 && window.innerWidth <= 1024;
  
  return isIPad || isAndroidTablet || isMediumScreen;
}

/**
 * 检测是否为Android设备
 */
export function isAndroid(): boolean {
  if (typeof window === 'undefined') return false;
  const userAgent = navigator.userAgent;
  return /Android/i.test(userAgent);
}

/**
 * 检测是否为iOS设备
 */
export function isIOS(): boolean {
  if (typeof window === 'undefined') return false;
  const userAgent = navigator.userAgent;
  return /iPad|iPhone|iPod/.test(userAgent) ||
    (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);
}

/**
 * 检测是否支持触摸
 */
export function isTouchDevice(): boolean {
  if (typeof window === 'undefined') return false;
  return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
}

/**
 * 获取安全区域尺寸
 */
export function getSafeAreaInset() {
  if (typeof window === 'undefined') {
    return { top: 0, bottom: 0, left: 0, right: 0 };
  }

  const style = getComputedStyle(document.documentElement);
  return {
    top: parseInt(style.getPropertyValue('safe-area-inset-top') || '0'),
    bottom: parseInt(style.getPropertyValue('safe-area-inset-bottom') || '0'),
    left: parseInt(style.getPropertyValue('safe-area-inset-left') || '0'),
    right: parseInt(style.getPropertyValue('safe-area-inset-right') || '0'),
  };
}
