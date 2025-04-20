var outsideHistogram = null;

function computeHistogram(times, bins) {
    var max = Math.max(...times);
    var min = Math.min(...times);
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
function renderTimeDetails(times) {
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
    console.log(Chart.defaults.color)
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

    // const labels = document.createElement('div');
    // labels.style.width = '3em';
    // histogram.counts.forEach((count, index) => {
    //     const labelBar = document.createElement('div');
    //     labelBar.classList = ["rowFlex"];
    //     const label = document.createElement('span');
    //     label.style.width = '4em';
    //     var leftEdge = histogram.leftEdges[index];
    //     label.textContent = `≥${Math.round(leftEdge) / 1000}`
    //     const bar = document.createElement('div');
    //     var count = count; // Convert milliseconds to seconds
    //     bar.style.width = `${count * scaleFactor}em`;
    //     bar.style.height = '1em';
    //     bar.style.backgroundColor = 'var(--primary)';
    //     bar.title = `Time ${index + 1}: ${count}`;
    //     labelBar.appendChild(label);
    //     labelBar.appendChild(bar);
    //     const countText = document.createElement("span");
    //     ratio = Math.round(count / total * 100) / 100
    //     countText.textContent = ``
    //     histogramContainer.appendChild(labelBar);
    // });

    

    // Append histogram to histogramDiv
    // histogramDiv.appendChild(histogramContainer);

    var timeList = document.getElementById("caseTimeDetailsTimes");

    var timeListStr = "";
    times.forEach((time, index) => {
        timeListStr += makeHtmlDisplayableTime(time) + (index < times.length - 1 ? ", " : "");
    });
    timeList.innerHTML = timeListStr;

}