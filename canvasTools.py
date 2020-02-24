import colorTools

import sys
import numpy

# BLACK reference
BLACK = numpy.array([0, 0, 0], numpy.uint32)


# converts a canvas into raw data for writing to a png
def toRawOutput(canvas):

    # converts the given canvas into a format that the PNG module can use to write a png
    simpleCanvas = numpy.array(canvas, numpy.uint8)
    transposedCanvas = numpy.transpose(simpleCanvas, (1, 0, 2))
    flippedColors = numpy.flip(transposedCanvas, 2)
    rawOutput = numpy.reshape(
        flippedColors, (canvas.shape[1], canvas.shape[0] * 3))
    return rawOutput


# Consider the target location using the given selection MODE
def considerPixelAt(canvas, targetCoordinates, targetColor, MODE):
    if (MODE == 0):
        return minimumSelection(canvas, targetCoordinates, targetColor)
    elif (MODE == 1):
        return averageSelection(canvas, targetCoordinates, targetColor)
    elif (MODE == 2):
        return fastAverageSelection(canvas, targetCoordinates, targetColor)
    else:
        return fastAverageSelection(canvas, targetCoordinates, targetColor)


# Gets the MINIMUM value of all the euclidianDistances beteen (valid neighbors of the target location) and (the target color)
def minimumSelection(canvas, targetCoordinates, targetColor):

    neighborDifferences = numpy.zeros(8, numpy.uint32)

    # Get all 8 neighbors, Loop over the 3x3 grid surrounding the location being considered
    count_differences = 0
    for i in range(3):
        for j in range(3):

            # this pixel is the location being considered;
            # it is not a neigbor, go to the next one
            if (i == 1 and j == 1):
                continue

            # calculate the neigbor's coordinates
            neighbor = numpy.array(
                [(targetCoordinates[0] - 1 + i), (targetCoordinates[1] - 1 + j)], numpy.uint32)

            # neighbor must be in the canvas
            neighborIsInCanvas = ((0 <= neighbor[0] < canvas.shape[0])
                                  and (0 <= neighbor[1] < canvas.shape[1]))
            
            neighborNotBlack = not numpy.array_equal(canvas[neighbor[0], neighbor[1]], BLACK)

            if (neighborIsInCanvas and neighborNotBlack):

                # get colDiff between the neighbor and target colors, add it to the list
                neigborColor = canvas[neighbor[0], neighbor[1]]

                # use the euclidian distance formula over [R,G,B] instead of [X,Y,Z]
                colorDifference = numpy.subtract(targetColor, neigborColor)
                differenceSquared = numpy.multiply(colorDifference, colorDifference)
                squaresSum = numpy.sum(differenceSquared)
                euclidianDistanceAprox = squaresSum

                neighborDifferences[count_differences] = euclidianDistanceAprox
                count_differences += 1

    # check if the considered pixel has at least one valid neighbor
    if (count_differences):
        # return the minimum difference of all the neighbors
        return numpy.min(neighborDifferences[0:count_differences])

    # if it has no valid neighbors, maximise its colorDiff
    else:
        return sys.maxsize


# Gets the AVERAGE value of all the euclidianDistances beteen (valid neighbors of the target location) and (the target color)
def averageSelection(canvas, targetCoordinates, targetColor):

    # Setup
    index = 0
    neighborDifferences = numpy.zeros(8, numpy.uint32)

    # Get neighbors
    # Don't consider BLACK pixels
    validNeighbors = removeNonColoredNeighbors(
        getNeighbors(canvas, targetCoordinates), canvas)

    for neighbor in validNeighbors:
        # get colDiff between the neighbor and target colors, add it to the list
        neigborColor = canvas[neighbor[0], neighbor[1]]
        euclidianDistanceAprox = colorTools.getColorDiff(targetColor, neigborColor)
        neighborDifferences[index] = euclidianDistanceAprox
        index += 1

    # check if the considered pixel has at least one valid neighbor
    if (index):

        return numpy.mean(neighborDifferences[0:index])

    # if it has no valid neighbors, maximise its colorDiff
    else:
        return sys.maxsize


# Gets the euclidianDistanceAprox beteen (the average color of valid neighbors of the target location) and (the target color)
def fastAverageSelection(canvas, targetCoordinates, targetColor):

    # Setup
    index = 0
    neigborhoodColor = BLACK

    # Get neighbors
    # Don't consider BLACK pixels
    validNeighbors = removeNonColoredNeighbors(
        getNeighbors(canvas, targetCoordinates), canvas)

    # sum up the color values from each neighbor
    for neighbor in validNeighbors:
        neigborhoodColor = numpy.add(
            canvas[neighbor[0], neighbor[1]], neigborhoodColor)
        index += 1

    # check if the considered pixel has at least one valid neighbor
    if (index):

        # divide through by the index to average the color
        indexArray = numpy.array([index, index, index], numpy.uint32)
        avgColor = numpy.divide(neigborhoodColor, indexArray)
        return colorTools.getColorDiff(avgColor, targetColor)

    # if it has no valid neighbors, maximise its colorDiff
    else:
        return sys.maxsize


# Gives all valid locations surrounding a given location
# A location is invalid only if it is outside the given canvas bonuds
def getNeighbors(canvas, targetCoordinates):

    # Setup
    index = 0
    neighbors = numpy.zeros([8, 2], numpy.uint32)

    # Get all 8 neighbors, Loop over the 3x3 grid surrounding the location being considered
    for i in range(3):
        for j in range(3):

            # this pixel is the location being considered;
            # it is not a neigbor, go to the next one
            if (i == 1 and j == 1):
                continue

            # calculate the neigbor's coordinates
            neighbor = numpy.array(
                [(targetCoordinates[0] - 1 + i), (targetCoordinates[1] - 1 + j)], numpy.uint32)

            # neighbor must be in the canvas
            neighborIsInCanvas = ((0 <= neighbor[0] < canvas.shape[0])
                                  and (0 <= neighbor[1] < canvas.shape[1]))
            if (neighborIsInCanvas):
                neighbors[index] = neighbor
                index += 1

    # if there are any valid, neighbors return them
    if (index):
        return neighbors[0:index]
    else:
        return numpy.array([])


# filters out colored locations from a given neighbor list
# i.e. neighbor color == BLACK
def removeColoredNeighbors(neighbors, canvas):

    # Setup
    index = 0
    filteredNeighbors = numpy.zeros([8, 2], numpy.uint32)

    # KEEP only NEIGHBORS that are NOT COLORED (color equal BLACK)
    for neighbor in neighbors:

        if (isNeighborBlack(neighbor, canvas)):
            filteredNeighbors[index] = neighbor
            index += 1

    # return the filtered neighbors or an empty list
    if (index):
        return filteredNeighbors[0:index]
    else:
        return numpy.array([])


# filters out non-colored locations from a given neighbor list
# i.e. neighbor color =/= BLACK
def removeNonColoredNeighbors(neighbors, canvas):

    # Setup
    index = 0
    filteredNeighbors = numpy.zeros([8, 2], numpy.uint32)

    # Keep only NEIGHBORS that are COLORED (color not equal BLACK)
    for neighbor in neighbors:

        if not (isNeighborBlack(neighbor, canvas)):
            filteredNeighbors[index] = neighbor
            index += 1

    # return the filtered neighbors or an empty list
    if (index):
        return filteredNeighbors[0:index]
    else:
        return numpy.array([])


# neighbor filter helper
def isNeighborBlack(neighbor, canvas):
    return numpy.array_equal(canvas[neighbor[0], neighbor[1]], BLACK)
