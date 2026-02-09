// 调试初始化脚本 - 用于诊断悬浮球显示问题
// 将此脚本通过 Console 运行来诊断问题

console.log('=== Floating Ball Debug ===');

// 1. 检查所有关键元素
const ballZone = document.getElementById('ball-zone');
const expansionZone = document.getElementById('expansion-zone');
const headerExpansion = document.getElementById('header-expansion');
const askerExpansion = document.getElementById('asker-expansion');
const ball = document.getElementById('floating-ball');

console.log('1. Elements:');
console.log('  - ballZone:', ballZone ? '✅ Found' : '❌ Not found');
console.log('  - expansionZone:', expansionZone ? '✅ Found' : '❌ Not found');
console.log('  - headerExpansion:', headerExpansion ? '✅ Found' : '❌ Not found');
console.log('  - ball:', ball ? '✅ Found' : '❌ Not found');

// 2. 检查 classList
console.log('\n2. ClassLists:');
console.log('  - expansionZone:', Array.from(expansionZone?.classList || []));
console.log('  - headerExpansion:', Array.from(headerExpansion?.classList || []));

// 3. 检查计算后的样式
console.log('\n3. Computed Styles:');
if (ballZone) {
  const ballZoneStyle = window.getComputedStyle(ballZone);
  console.log('  - ballZone width:', ballZoneStyle.width);
  console.log('  - ballZone height:', ballZoneStyle.height);
  console.log('  - ballZone display:', ballZoneStyle.display);
}

if (expansionZone) {
  const expansionStyle = window.getComputedStyle(expansionZone);
  console.log('  - expansionZone display:', expansionStyle.display);
  console.log('  - expansionZone width:', expansionStyle.width);
}

if (headerExpansion) {
  const headerStyle = window.getComputedStyle(headerExpansion);
  console.log('  - headerExpansion display:', headerStyle.display);
}

// 4. 检查窗口尺寸
console.log('\n4. Window Size:');
console.log('  - window.innerWidth:', window.innerWidth);
console.log('  - window.innerHeight:', window.innerHeight);
console.log('  - body clientWidth:', document.body.clientWidth);
console.log('  - body clientHeight:', document.body.clientHeight);

// 5. 强制隐藏展开区域
console.log('\n5. Force Hide Expansion Zones:');
if (expansionZone && !expansionZone.classList.contains('hidden')) {
  expansionZone.classList.add('hidden');
  console.log('  ✅ Added "hidden" to expansionZone');
} else if (expansionZone) {
  console.log('  ✅ expansionZone already has "hidden"');
}

if (headerExpansion && !headerExpansion.classList.contains('hidden')) {
  headerExpansion.classList.add('hidden');
  console.log('  ✅ Added "hidden" to headerExpansion');
} else if (headerExpansion) {
  console.log('  ✅ headerExpansion already has "hidden"');
}

// 6. 强制设置 Ball Zone 宽度
console.log('\n6. Force Ball Zone Width:');
if (ballZone) {
  ballZone.style.width = '64px';
  ballZone.style.justifyContent = 'center';
  console.log('  ✅ Set ballZone width to 64px');
}

console.log('\n=== Debug Complete ===');
console.log('Expected: Only see a 64x64 green circle (Ball)');
console.log('If you still see Header buttons, the window size is wrong!');
