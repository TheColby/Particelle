#pragma once

#include <juce_audio_utils/juce_audio_utils.h>
#include "obelisk_m0.h"

#include <atomic>
#include <vector>

class ObeliskProcessor final : public juce::AudioProcessor {
public:
    ObeliskProcessor();
    ~ObeliskProcessor() override;

    const juce::String getName() const override { return "Obelisk M0"; }
    void prepareToPlay(double sampleRate, int samplesPerBlock) override;
    void releaseResources() override;
    void processBlock(juce::AudioBuffer<float>&, juce::MidiBuffer&) override;
    bool isBusesLayoutSupported(const BusesLayout&) const override;
    bool acceptsMidi() const override { return true; }
    bool producesMidi() const override { return false; }
    bool isMidiEffect() const override { return false; }
    double getTailLengthSeconds() const override { return 2.0; }
    bool hasEditor() const override { return true; }
    juce::AudioProcessorEditor* createEditor() override;

    int getNumPrograms() override { return 3; }
    int getCurrentProgram() override { return program.load(); }
    void setCurrentProgram(int index) override;
    const juce::String getProgramName(int index) override;
    void changeProgramName(int, const juce::String&) override {}
    void getStateInformation(juce::MemoryBlock&) override;
    void setStateInformation(const void*, int) override;

private:
    static juce::AudioProcessorValueTreeState::ParameterLayout makeParameters();
    void render(juce::AudioBuffer<float>&, int start, int count);

    juce::AudioProcessorValueTreeState state;
    std::atomic<int> program { 0 };
    ObeliskM0Handle* engine = nullptr;
    std::vector<float> interleaved;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(ObeliskProcessor)
};
