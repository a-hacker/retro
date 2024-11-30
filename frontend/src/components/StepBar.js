// src/components/StepBar.js

import React, { useEffect } from 'react';
import PropTypes from 'prop-types';
import { Grid2, Stepper, Step, StepLabel, Box, IconButton, Tooltip } from '@mui/material';
import { styled } from '@mui/system';
import ChevronLeftIcon from '@mui/icons-material/ChevronLeft';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

// Define the steps of the retrospective process
const steps = ['Writing', 'Grouping', 'Voting', 'Reviewing'];

// Styled container to fix the Stepper at the bottom
const FixedStepperContainer = styled(Box)(({ theme }) => ({
  position: 'fixed',
  bottom: 0,
  left: 0,
  right: 0,
  backgroundColor: '#f5f5f5', // Light grey background
  padding: '10px 0',  // theme.spacing(1, 2),
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center', // Center content initially
  boxShadow: '0 -2px 5px rgba(0,0,0,0.1)', // Subtle shadow for elevation
}));

const StepBar = ({ currentStep, handleRetroStepClick, subscribeToStep }) => {
  useEffect(() => subscribeToStep(), []);

  // Find the index of the current step
  const activeStep = steps.indexOf(currentStep);

  // Determine if buttons should be disabled
  const isFirstStep = activeStep === 0;
  const isLastStep = activeStep === steps.length - 1;

  const handleStep = (step) => () => {
    console.log(step);
  };

  return (
    <FixedStepperContainer>
      <Grid2 container sx={{width: '90%'}}>
        <Grid2 size={1} display="flex" justifyContent="right" alignItems="right">
          <Tooltip title="Previous Step" placement="top">
            <span> {/* Span wrapper to allow Tooltip on disabled IconButton */}
              <IconButton
                disabled={isFirstStep}
                onClick={() => handleRetroStepClick(steps[activeStep - 1])}
                aria-label="previous step"
                color="primary"
              >
                <ChevronLeftIcon />
              </IconButton>
            </span>
          </Tooltip>
        </Grid2>

        <Grid2 size={"grow"}>
          {/* Stepper */}
          <Stepper activeStep={activeStep} alternativeLabel>
            {steps.map((label, index) => (
              <Step key={label}>
                <StepLabel color="inherit" onClick={handleStep(index)}>
                  {label}
                </StepLabel>
              </Step>
            ))}
          </Stepper>
        </Grid2>

        <Grid2 size={1} display="flex" justifyContent="left" alignItems="left">
          <Tooltip title="Next Step" placement="top">
            <span> {/* Span wrapper to allow Tooltip on disabled IconButton */}
              <IconButton
                disabled={isLastStep}
                onClick={() => handleRetroStepClick(steps[activeStep + 1])}
                aria-label="next step"
                color="primary"
              >
                <ChevronRightIcon />
              </IconButton>
            </span>
          </Tooltip>
        </Grid2>
      </Grid2>
  </FixedStepperContainer>
  );
};

// PropTypes for type checking
StepBar.propTypes = {
  currentStep: PropTypes.string.isRequired,
};

export default StepBar;
