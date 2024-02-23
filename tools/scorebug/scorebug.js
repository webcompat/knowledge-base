async function urlRank(url) {
  const parsedUrl = new URL(url);
  let targetDomain = parsedUrl.host;
  let rank = null;
  while (targetDomain.includes(".")) {
    const resp = await fetch(`https://tranco-list.eu/api/ranks/domain/${encodeURI(targetDomain)}`);
    const data = await resp.json();
    if (data && data.ranks.length) {
      rank = data.ranks[0].rank;
      break;
    }
    const [first, ...rest] = targetDomain.split(".");
    targetDomain = rest.join(".");
    // Avoid rate limits
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  return rank;
}

function getSeverity(data, siteRank) {
  const impactScore = parseInt(data.impact);

  const affectsModifier = parseFloat(data.affects);

  const platformModifier = Array.from(Object.values(data.platforms)).reduce((prev, current) => parseFloat(current) + prev, 0);

  const severityScore = Math.round(impactScore * affectsModifier * platformModifier);
  let severity = "S4";
  if (severityScore > 50) {
    if (siteRank && siteRank <= 500) {
      severity = "S1";
    } else {
      severity = "S2";
    }
  } else if (severityScore > 25) {
    severity = "S3";
  } else {
    severity = "S4";
  }
  return {
    severity,
    severityScore
  };
}

function getPriority(severity, siteRank) {
  const severityRank = parseInt(severity.severity[1]);
  let priority;
  let priorityScore;
  if (siteRank === null || siteRank > 100000) {
    priority = "P3";
    priorityScore = 1;
  } else if (siteRank > 10000) {
    priorityScore = 2;
    if (severityRank >= 3) {
      priority = "P3";
    } else {
      priority = "P2";
    }
  } else if (siteRank > 1000) {
    priorityScore = 5;
    if (severityRank === 4) {
      priority = "P3";
    } else if (severityRank === 3) {
      priority = "P2";
    } else {
      priority = "P1";
    }
  } else {
    priorityScore = 10;
    if (severityRank >= 3) {
      priority = "P2";
    } else {
      priority = "P1";
    }
  }
  return {priority, priorityScore};
}

function readData() {
  const platforms = {};
  ["windows", "mac", "linux", "mobile"].forEach(platform => {
    const element = document.getElementById(`platform-${platform}`);
    platforms[platform] = element.checked ? element.value : "0";
  });
  return {
    bugId: document.getElementById("bug-id").value,
    url: document.getElementById("url").value,
    platforms,
    impact: document.getElementById("impact").value,
    affects: document.getElementById("affects").value,
  };
}

function resetOutput() {
  document.getElementById("output").hidden = true;
}

function setOutput(outputData) {
  document.getElementById("domain-rank").value = outputData.rank;
  document.getElementById("severity-score").value = outputData.severityScore;
  document.getElementById("priority-score").value = outputData.priorityScore;
  document.getElementById("severity").value = outputData.severity;
  document.getElementById("priority").value = outputData.priority;
  document.getElementById("score").value = outputData.score;
  document.getElementById("output").hidden = false;
}

async function score() {
  resetOutput();
  const data = readData();
  const rank = await urlRank(data.url);
  const severity = getSeverity(data, rank);
  const priority = getPriority(severity, rank);
  const outputData = {
    rank: rank,
    ...severity,
    ...priority
  };
  outputData.score = outputData.severityScore * outputData.priorityScore;
  setOutput(outputData);
}
