#include "PluginProcessor.h"

#include <cmath>
#include <cstdio>

int main() {
    ObeliskProcessor synth;
    synth.prepareToPlay(48000.0, 256);
    synth.setCurrentProgram(1);
    if (synth.getProgramName(synth.getCurrentProgram()) != "Glass Pressure")
        return 1;

    juce::MemoryBlock state;
    synth.getStateInformation(state);
    ObeliskProcessor restored;
    restored.setStateInformation(state.getData(), static_cast<int>(state.getSize()));
    if (restored.getCurrentProgram() != 1)
        return 2;

    juce::AudioBuffer<float> audio(2, 256);
    juce::MidiBuffer midi;
    midi.addEvent(juce::MidiMessage::noteOn(2, 60, static_cast<juce::uint8>(100)), 32);
    synth.processBlock(audio, midi);
    for (int channel = 0; channel < 2; ++channel)
        for (int frame = 0; frame < 32; ++frame)
            if (audio.getSample(channel, frame) != 0.0f)
                return 3;
    bool audible = false;
    for (int channel = 0; channel < 2; ++channel)
        for (int frame = 32; frame < 256; ++frame) {
            const float sample = audio.getSample(channel, frame);
            if (!std::isfinite(sample))
                return 4;
            audible |= std::abs(sample) > 0.0f;
        }
    if (!audible)
        return 5;
    std::puts("Obelisk M0: preset/state and sample-offset MIDI/audio smoke passed");
    return 0;
}
