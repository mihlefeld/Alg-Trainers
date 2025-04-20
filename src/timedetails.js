var outsideHistogram = null;

function sumArray(times) {
    return times.reduce((total, current) => total + current, 0);
}

function meanArray(times) {
    return sumArray(times) / times.length;
}

function meanStdArray(times) {
    var mean = meanArray(times);
    var variance = sumArray(times.map((x) => (x - mean) * (x - mean))) / (times.length - 1)
    return {"std": Math.sqrt(variance), "mean": mean};
}

function computeHistogram(times, maxBins) {
    const bins = Math.min(maxBins, times.length);
    var meanStd = meanStdArray(times);
    var arrMax = Math.max(...times);
    var arrMin = Math.min(...times);
    var min = Math.max(meanStd.mean - meanStd.std, arrMin);
    var max = Math.min(meanStd.mean + meanStd.std, arrMax);
    var diff = max - min;
    var counts = new Array(bins).fill(0);
    var binLeftEdges = [...counts.keys()].map((index) => {return min + diff / bins * index});
    times.forEach((time, _) => {
        var binIndex = Math.floor((time - min) / diff * bins);
        binIndex = Math.max(Math.min(bins - 1, binIndex), 0);
        counts[binIndex] += 1;
    })
    return {"leftEdges": binLeftEdges, "counts": counts}
}
// Function to render a histogram and a list of times into the caseTimeDetails histogramDiv
function renderTimeDetails(caseNum) {
    if (caseNum < 0) {
        var times = window.timesArray;
    } else {
        var times = window.timesArray.filter((result) => result["case"] == caseNum);
    }
    var timesMs = times.map(time => time.ms); // Extract milliseconds from the times array

    // Calculate histogram data
    const histogram = computeHistogram(timesMs, 10);
    const maxCount = Math.max(...histogram.counts);
    const scaleFactor = 20 / maxCount;
    var total = times.length;

    var labels = new Array();
    histogram.leftEdges.forEach((time, index) => {
        var seconds = Math.round(time) / 1000;
        if (index == histogram.leftEdges.length - 1) {
            labels.push(`>${seconds}`)
            return
        }
        var secondsHigh = Math.round(histogram.leftEdges[index + 1]) / 1000;
        labels.push(`${seconds}-${secondsHigh}`)
    });

    Chart.defaults.color = window.getComputedStyle(document.body).getPropertyValue("--text");
    Chart.defaults.backgroundColor = window.getComputedStyle(document.body).getPropertyValue("--primaryBGHover");
    Chart.defaults.borderColor = window.getComputedStyle(document.body).getPropertyValue("--primaryBG");
    const ctx = document.getElementById("timeHistogram");
    if (outsideHistogram === null) {        
        outsideHistogram = new Chart(ctx, {
            type: 'bar',
            data: {
            labels: [],
            datasets: [{
                label: 'Count',
                data: [],
                borderWidth: 1,
            }]
            },
            options: {
            scales: {
                y: {
                    beginAtZero: true,
                    border: {
                        display: false
                    },
                    grid: {
                        display: false
                    }
                },
                x: {
                    beginAtZero: true,
                    border: {
                        display: true
                    },
                    grid: {
                        display: false
                    }
                }
            }
            }
        });
    } 

    outsideHistogram.data.labels = labels;
    outsideHistogram.data.datasets.forEach((dataset) => {
        dataset.data = histogram.counts;
    });
    outsideHistogram.update();


    var timeList = document.getElementById("caseTimeDetailsTimes");

    var timeListStr = "";
    times.forEach((time, index) => {
        timeListStr += makeHtmlDisplayableTime(time) + (index < times.length - 1 ? ", " : "");
    });
    timeList.innerHTML = timeListStr;

}