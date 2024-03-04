const ALL_PLATFORMS = getCategoryValues("platform", "input");

const ALL_IMPACT = getCategoryValues("impact", "option");

const ALL_AFFECTS = getCategoryValues("affects", "option");

function getCategoryValues(prefix, elemType) {
  console.log(`#${prefix} ${elemType}`, document.querySelectorAll(`#${prefix} ${elemType}`));
  return Array.from(document.querySelectorAll(`#${prefix} ${elemType}`)).map(elem => {
    const [gotPrefix, ...rest] = elem.id.split("-");
    if (gotPrefix != prefix) {
      console.warn("Unexpected element", elem);
      return null;
    }
    return rest.join("-");
  }).filter(x => x);
}

function getSeverity(data, siteRank) {
  const impactScore = parseInt(data.impact.value);

  const affectsModifier = parseFloat(data.affects.value);

  const platformModifier = Array.from(Object.values(data.platforms)).reduce((prev, current) => parseFloat(current.value) + prev, 0);

  const severityScore = Math.round(impactScore * affectsModifier * platformModifier);
  let severity = "S4";
  if (severityScore >= 100) {
    if (siteRank && siteRank <= 100) {
      severity = "S1";
    } else {
      severity = "S2";
    }
  } else if (severityScore > 50) {
    severity = "S2";
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

function getSelectedId(selectElem) {
  const optionId = selectElem.selectedOptions[0].id;
  const [prefix, ...value] = optionId.split("-");
  return value.join("-");
}

function readData() {
  const platforms = {};
  ALL_PLATFORMS.forEach(platform => {
    const element = document.getElementById(`platform-${platform}`);
    platforms[platform] = {name: platform,
                           value: element.checked ? element.value : "0",
                           included: element.checked};
  });
  return {
    bugId: document.getElementById("bug-id").value,
    url: document.getElementById("url").value,
    platforms,
    impact: {name: getSelectedId(document.getElementById("impact")),
             value: document.getElementById("impact").value},
    affects: {name: getSelectedId(document.getElementById("affects")),
              value: document.getElementById("affects").value},
  };
}

function getUserStory(data) {
  console.log(data, Array.from(Object.values(data.platforms)).filter(x => x.included));
  return `platform:${Array.from(Object.values(data.platforms)).filter(x => x.included).map(x => x.name).join(",")}
impact:${data.impact.name}
affects:${data.affects.name}
`;
}

function resetOutput() {
  document.getElementById("output").hidden = true;
}

function setOutput(outputData) {
  document.getElementById("domain").value = outputData.rankedDomain;
  document.getElementById("domain-rank").value = outputData.rank;
  document.getElementById("severity-score").value = outputData.severityScore;
  document.getElementById("priority-score").value = outputData.priorityScore;
  document.getElementById("severity").value = outputData.severity;
  document.getElementById("priority").value = outputData.priority;
  document.getElementById("score").value = outputData.score;
  document.getElementById("user-story").value = outputData.userStory;
  document.getElementById("output").hidden = false;
}

function copyElemText(elemId) {
  const elem = document.getElementById(elemId);
  navigator.clipboard.writeText(elem.textContent);
}

async function score() {
  resetOutput();
  const data = readData();
  const rank = await rankUrl(data.url);
  const severity = getSeverity(data, rank.rank);
  const priority = getPriority(severity, rank.rank);
  const userStory = getUserStory(data);
  const outputData = {
    ...rank,
    ...severity,
    ...priority,
    userStory
  };
  outputData.score = outputData.severityScore * outputData.priorityScore;
  setOutput(outputData);
}

function parseUserStory(userStory) {
  const lines = userStory.split(/\r?\n|\r|\n/g);
  const rv = {};
  for (const line of lines) {
    const [prefix, ...rest] = line.split(":");
    const data = rest.join(":").trim();
    if (prefix === "platform") {
      const foundPlatforms = [];
      const [...dataPlatforms] = data.split(",");
      for (let platform of dataPlatforms) {
        platform = platform.trim();
        if (ALL_PLATFORMS.includes(platform)) {
          foundPlatforms.push(platform);
        }
      }
      if (foundPlatforms.length) {
        rv.platforms = foundPlatforms;
      }
    } else if (prefix === "impact") {
      if (ALL_IMPACT.includes(data)) {
        rv.impact = data;
      }
    } else if (prefix === "affects") {
        if (ALL_AFFECTS.includes(data)) {
          rv.affects = data;
        }
    }
  }
  return rv;
}

function setFieldsFromBug(data) {
  document.getElementById("url").value = data.url;
  let platforms;
  if (data.platforms) {
    ALL_PLATFORMS.forEach(platform => {
      const element = document.getElementById(`platform-${platform}`);
      element.checked = data.platforms.includes(platform);
    });
  }
  if (data.impact) {
    document.getElementById(`impact-${data.impact}`).selected = true;
  }
  if (data.affects) {
    document.getElementById(`affects-${data.affects}`).selected = true;
  }
}

async function updateFromBug() {
  let bugNumber = parseInt(document.getElementById("bug-id").value);

  const resp = await fetch(`https://bugzilla.mozilla.org/rest/bug/${bugNumber}`);
  const bugsData = await resp.json();
  const bugData = bugsData.bugs[0];

  if (bugData.product != "Web Compatibility") {
    console.log(`Bug ${bugNumber} is not a Web Compatibility product bug`);
    return;
  }

  const data = {url: bugData.url};
  if (bugData.cf_user_story) {
    Object.assign(data, parseUserStory(bugData.cf_user_story));
  }
  data.url = bugData.url;
  if (!data.platforms) {
    let platforms;
    if (bugData.component == "Desktop") {
      platforms = ["windows", "mac", "linux"];
    } else if (bugData.component == "Mobile") {
      platforms = ["android"];
    }
  }
  setFieldsFromBug(data);
}
